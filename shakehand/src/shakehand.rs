use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use std::collections::{BTreeMap, BTreeSet};

use crate::analyzer::{
    TomlFile, TranslationEntry, extract_params, key_to_ident, lang_to_variant, path_to_mod_name,
    replace_params_with_format,
};

/// Generate the language enum (and functions `lang()` / `set_lang()`)
fn generate_languages_enum(
    all_languages: &BTreeSet<String>,
    fallback: &str,
    locale_path: &str,
) -> TokenStream2 {
    let variants_info: Vec<(Ident, String)> = all_languages
        .iter()
        .map(|lang| {
            let name = lang_to_variant(lang);
            (format_ident!("{}", name), lang.clone())
        })
        .collect();

    let enum_doc = format!("All language files present under \"{}\"", locale_path);

    if variants_info.is_empty() {
        return quote! {
            /// This constant stores the discriminant of the current language variant.
            /// It is initialized at program start by reading the locale or a config file.
            pub static __SHAKE_HAND_LANG: std::sync::atomic::AtomicU8 =
                std::sync::atomic::AtomicU8::new(0u8);

            #[derive(Debug, Default, Clone, Copy)]
            #[repr(u8)]
            #[doc = #enum_doc]
            pub enum Languages {}

            #[inline(always)]
            /// Get the current language from the global static variable
            pub fn lang() -> Languages {
                panic!("shakehand: no locale files found")
            }

            #[inline(always)]
            /// Set the current language in the global static variable
            pub fn set_lang(_lang: Languages) {
                panic!("shakehand: no locale files found")
            }
        };
    }

    let fallback_idx = all_languages
        .iter()
        .position(|l| l == fallback)
        .unwrap_or(0);

    let fallback_ident = &variants_info[fallback_idx].0;

    // List of enum variants with doc comments, fallback variant gets #[default]
    let enum_variants: Vec<TokenStream2> = variants_info
        .iter()
        .enumerate()
        .map(|(i, (ident, raw))| {
            if i == fallback_idx {
                quote! {
                    #[doc = #raw]
                    #[default]
                    #ident,
                }
            } else {
                quote! {
                    #[doc = #raw]
                    #ident,
                }
            }
        })
        .collect();

    // lang() match arms (match u8 values as returned by AtomicU8::load)

    let lang_match_arms: Vec<TokenStream2> = variants_info
        .iter()
        .enumerate()
        .map(|(i, (ident, _))| {
            let idx = i as u8;
            quote! { #idx => Languages::#ident, }
        })
        .collect();

    quote! {
        /// This constant stores the discriminant of the current language variant.
        /// It is initialized at program start by reading the locale or a config file.
        pub static __SHAKE_HAND_LANG: std::sync::atomic::AtomicU8 =
            std::sync::atomic::AtomicU8::new(#fallback_idx as u8);

        #[derive(Debug, Default, Clone, Copy)]
        #[repr(u8)]
        #[allow(non_camel_case_types)]
        #[doc = #enum_doc]
        pub enum Languages {
            #(#enum_variants)*
        }

        /// Get the current language from the global static variable
        #[inline(always)]
        pub fn lang() -> Languages {
            match __SHAKE_HAND_LANG.load(std::sync::atomic::Ordering::Relaxed) {
                #(#lang_match_arms)*
                _ => Languages::#fallback_ident,
            }
        }

        /// Set the current language in the global static variable
        #[inline(always)]
        pub fn set_lang(lang: Languages) {
            __SHAKE_HAND_LANG.store(lang as u8, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

/// Generate a `format!(fmt_str, args…)` expression for a value that has parameters
fn make_format_expr(value: &str) -> TokenStream2 {
    let fmt_str = replace_params_with_format(value);
    let lang_params = extract_params(value);
    let format_args: Vec<TokenStream2> = lang_params
        .iter()
        .map(|p| {
            let var = format_ident!("{}", just_fmt::snake_case!(p));
            // 取出 .as_ref() 后的变量值
            quote! { #var }
        })
        .collect();
    quote! { format!(#fmt_str, #(#format_args),*) }
}

/// Generate match arms for a single entry (arms for languages with values) and a `_ =>` catch-all (fallback)
fn make_match_arms(
    entry: &TranslationEntry,
    all_available: &BTreeSet<String>,
    fallback: &str,
) -> (Vec<TokenStream2>, TokenStream2) {
    let mut arms: Vec<TokenStream2> = Vec::new();
    let mut found_fallback = false;

    let mut fallback_arm = if entry.has_params {
        quote! { _ => ::std::string::String::new(), }
    } else {
        quote! { _ => "", }
    };

    for lang in all_available {
        let value = entry.values.get(lang.as_str());
        let variant_name = format_ident!("{}", lang_to_variant(lang));
        let is_fallback = lang == fallback;

        match value {
            Some(v) if entry.has_params => {
                let body = make_format_expr(v);
                let arm = quote! { Languages::#variant_name => #body, };
                if is_fallback {
                    found_fallback = true;
                    fallback_arm = quote! { _ => #body, };
                }
                arms.push(arm);
            }
            Some(v) => {
                let arm = quote! { Languages::#variant_name => #v, };
                if is_fallback {
                    found_fallback = true;
                    fallback_arm = quote! { _ => #v, };
                }
                arms.push(arm);
            }
            None => {}
        }
    }

    // When the fallback language doesn't have a value for this key, use the first available language as a catch-all
    if !found_fallback && let Some(first_val) = entry.values.values().next() {
        if entry.has_params {
            let body = make_format_expr(first_val);
            fallback_arm = quote! { _ => #body, };
        } else {
            fallback_arm = quote! { _ => #first_val, };
        }
    }

    (arms, fallback_arm)
}

/// Generate a method for a single translation entry
fn generate_entry_method(
    entry: &TranslationEntry,
    all_languages: &BTreeSet<String>,
    fallback: &str,
) -> TokenStream2 {
    let method_name = format_ident!("{}", key_to_ident(&entry.key));
    let key_str = format!("Key \"{}\"", entry.key);

    // Doc table showing each language's value for this key
    let mut lang_rows: Vec<TokenStream2> = Vec::new();
    // Table header
    lang_rows.push(quote! { #[doc = "|Language|Value|"] });
    lang_rows.push(quote! { #[doc = "|-|-|"] });
    for lang in all_languages.iter() {
        let val = entry
            .values
            .get(lang.as_str())
            .map(|s| s.as_str())
            .unwrap_or("(NO TRANSLATION)");
        let row = format!("|**{}**|*\"{}\"*|", lang, val);
        lang_rows.push(quote! { #[doc = #row] });
    }
    let lang_docs = lang_rows;

    // Parameter name conflict: compile error + deprecated function
    if entry.params_conflict {
        let err_msg = format!(
            "shakehand: key `{}` has inconsistent parameter names across languages",
            entry.key,
        );
        let panic_msg = format!(
            "shakehand: key `{}` has inconsistent parameter names across languages, fix the .toml file",
            entry.key,
        );
        return quote! {
            ::core::compile_error!(#err_msg);

            #[deprecated(note = "parameter mismatch across languages, fix the .toml file")]
            #[doc = #key_str]
            ///
            #(#lang_docs)*
            #[must_use]
            pub fn #method_name () -> ! {
                panic!(#panic_msg)
            }
        };
    }

    // Only generate match arms for languages that have a translation; missing ones fall through to `_ =>`
    let (match_arms, catch_all) =
        make_match_arms(entry, &entry.values.keys().cloned().collect(), fallback);

    if entry.has_params {
        let params_with_type: Vec<TokenStream2> = entry
            .params
            .iter()
            .map(|p| {
                let name = format_ident!("{}", just_fmt::snake_case!(p));
                quote! { #name: impl AsRef<str> }
            })
            .collect();

        let param_bindings: Vec<TokenStream2> = entry
            .params
            .iter()
            .map(|p| {
                let name = format_ident!("{}", just_fmt::snake_case!(p));
                quote! { let #name = #name.as_ref(); }
            })
            .collect();

        let param_docs: Vec<TokenStream2> = entry
            .params
            .iter()
            .map(|p| {
                let doc = format!("- `{}`", p);
                quote! { #[doc = #doc] }
            })
            .collect();

        quote! {
            #[inline(always)]
            #[doc = #key_str]
            ///
            #(#lang_docs)*
            ///
            /// # Parameters
            #(#param_docs)*
            #[must_use]
            pub fn #method_name (#(#params_with_type),*) -> String {
                #(#param_bindings)*
                match lang() {
                    #(#match_arms)*
                    #catch_all
                }
            }
        }
    } else {
        quote! {
            #[inline(always)]
            #[doc = #key_str]
            ///
            #(#lang_docs)*
            #[must_use]
            pub fn #method_name () -> &'static str {
                match lang() {
                    #(#match_arms)*
                    #catch_all
                }
            }
        }
    }
}

/// Generate a struct and its impl block for a single toml file
fn generate_struct(
    toml_file: &TomlFile,
    all_languages: &BTreeSet<String>,
    locale_path: &str,
    fallback: &str,
) -> TokenStream2 {
    let struct_name = format_ident!("{}", toml_file.struct_name);
    let methods: Vec<TokenStream2> = toml_file
        .entries
        .iter()
        .map(|entry| generate_entry_method(entry, all_languages, fallback))
        .collect();

    let struct_name_str = toml_file.struct_name.as_str();

    // Count how many keys each language has, for the table
    let mut lang_counts: Vec<(String, usize)> = all_languages
        .iter()
        .map(|lang| {
            let count = toml_file
                .entries
                .iter()
                .filter(|e| e.values.contains_key(lang.as_str()))
                .count();
            (lang.clone(), count)
        })
        .collect();
    lang_counts.sort_by(|a, b| a.1.cmp(&b.1).reverse());

    // Table rows
    let mut count_rows: Vec<TokenStream2> = Vec::new();
    count_rows.push(quote! { #[doc = "|Language|Count|"] });
    count_rows.push(quote! { #[doc = "|-|-|"] });
    for (lang, count) in &lang_counts {
        let row = format!("|**{}**|{}|", lang, count);
        count_rows.push(quote! { #[doc = #row] });
    }

    let path_doc = format!(
        "Language information from file `{}/{}.toml`",
        locale_path, struct_name_str
    );

    quote! {
        #[doc = concat!("# ", #struct_name_str)]
        ///
        #[doc = #path_doc]
        ///
        #(#count_rows)*
        pub struct #struct_name;

        impl #struct_name {
            #(#methods)*
        }
    }
}

/// Generate the complete module code
pub fn generate_module(
    files: Vec<TomlFile>,
    all_languages: BTreeSet<String>,
    fallback: String,
    locale_path: &str,
) -> TokenStream2 {
    let lang_enum = generate_languages_enum(&all_languages, &fallback, locale_path);

    // Group by module path
    let mut root_files: Vec<&TomlFile> = Vec::new();
    let mut sub_modules: BTreeMap<String, Vec<&TomlFile>> = BTreeMap::new();

    for f in &files {
        if f.module_path.is_empty() {
            root_files.push(f);
        } else {
            let mod_name = f.module_path[0].clone();
            sub_modules.entry(mod_name).or_default().push(f);
        }
    }

    // Generate root-level structs
    let root_structs: Vec<TokenStream2> = root_files
        .iter()
        .map(|f| generate_struct(f, &all_languages, locale_path, &fallback))
        .collect();

    // Generate sub-modules
    let sub_mods: Vec<TokenStream2> = sub_modules
        .iter()
        .map(|(mod_name, mod_files): (&String, &Vec<&TomlFile>)| {
            let mod_ident = format_ident!("{}", path_to_mod_name(mod_name));
            let sub_structs: Vec<TokenStream2> = mod_files
                .iter()
                .map(|f| {
                    let fixed_file = TomlFile {
                        module_path: f.module_path[1..].to_vec(),
                        struct_name: f.struct_name.clone(),
                        entries: f.entries.clone(),
                        all_languages: f.all_languages.clone(),
                    };
                    generate_struct(&fixed_file, &all_languages, locale_path, &fallback)
                })
                .collect();

            quote! {
                pub mod #mod_ident {
                    #(#sub_structs)*
                }
            }
        })
        .collect();

    quote! {
        #lang_enum

        #(#root_structs)*

        #(#sub_mods)*
    }
}
