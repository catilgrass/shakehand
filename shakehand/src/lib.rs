//! "Shake Hand!"
//!
//! > Let different languages shake hands with each other! 🤝
//!
//! This is a **purely compile-time** Rust internationalization library. **All** localized strings are fully embedded into the binary at compile time, with only minimal runtime overhead.
//!
//! # How to Use?
//!
//! ## 1. Add Dependency
//!
//! Add the following to your `Cargo.toml`:
//!
//! ```toml
//! shakehand = "0.1"
//! ```
//!
//! ## 2. Write Configuration Files
//!
//! Create a directory under your project to serve as the root for translations, e.g. `./locale/`.
//! Create any `toml` file inside this directory, e.g. `./locale/global.toml`, representing a translation file.
//!
//! In the file, group translations by language section. The key is the translation key, and the value is the translated text.
//! Text with parameters uses `%{parameter_name}` as placeholders, where the parameter name directly maps to the generated function's parameter name:
//!
//! ```toml
//! [en]
//! world = "world"
//! greeting = "Hello, %{someone}!"
//!
//! [zh_CN]
//! world = "世界"
//! greeting = "你好，%{someone}！"
//! ```
//!
//! ## 3. Load Translations
//!
//! In your Rust code, use `shakehand::locale!` to hardcode the entire directory into a module:
//!
//! ```rust
//! pub mod translation {
//!     shakehand::locale!("./test-locale", fallback = "en");
//! }
//! ```
//!
//! The generated module will contain:
//! - A `Languages` enum listing all languages
//! - A `set_lang` function for switching the current language
//! - A unit struct `Global` (named after `global.toml`), whose associated functions are the translations for each key
//!
//! ## 4. Call Translations
//!
//! Call translations just like ordinary functions, passing parameters by placeholder name:
//!
//! ```rust
//! use crate::translation::{Global, Languages, set_lang};
//!
//! fn main() {
//!     set_lang(Languages::en);
//!     let greeting = Global::greeting("World");
//!     println!("{}", greeting);  // Hello, World!
//!
//!     set_lang(Languages::zh_CN);
//!     let greeting = Global::greeting("世界");
//!     println!("{}", greeting);  // 你好，世界！
//! }
//!
//! # pub mod translation {
//! #     shakehand::locale!("./test-locale", fallback = "en");
//! # }
//! ```
//!
//! You can also pass the return value of another translation as a parameter, since each parameterless translation function returns a `&'static str`, completely allocation-free:
//!
//! ```rust
//! # use shakehand::locale;
//! # use a::Global;
//! # pub mod a { shakehand::locale!("./test-locale"); };
//! # a::set_lang(a::Languages::en);
//! let greeting = Global::greeting(Global::world());
//! ```
//!
//! ## 5. Fallback Chain (Optional)
//!
//! You can configure a **fallback chain** in your `Cargo.toml` under `[package.metadata.shakehand]`.
//! When a language has no translation for a key, the system will automatically walk the chain
//! to find the nearest language that has one.
//!
//! ```toml
//! [package.metadata.shakehand]
//! # For any language not explicitly listed, fall back to English
//! fallback.other = "en"
//! # Specific fallback rules
//! fallback.zh_HK = "zh_CN"   # Hong Kong Trad -> Simplified Chinese
//! fallback.zh_TW = "zh_CN"   # Taiwan Trad -> Simplified Chinese
//! fallback.zh_CN = "en"      # Simplified Chinese -> English
//! fallback.it = "fr"         # Italian -> French
//! ```
//!
//! The chain resolution is **at runtime via a generated `FallbackSolver`**:
//! - Each language has a `try_fallback_once()` step defined in a compile-time generated match.
//! - Translation functions loop through the chain until a language with the value is found.
//! - `fallback.other` is the ultimate fallback (falls back to `locale!`'s `fallback` parameter if unset).
//! - All languages in the chain are automatically added to the `Languages` enum, even if they
//!   have no translations in the locale files.
//!
//! # Contributing
//!
//! Directly open a PR to the [repository](https://github.com/catilgrass/shakehand) and mention [@Weicao-CatilGrass](https://github.com/Weicao-CatilGrass).
//!
//! # License
//!
//! MIT or Apache 2.0

use std::path::Path;

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod analyzer;
mod shakehand;

/// `locale!` macro: Generate an i18n module from toml files at compile time
///
/// # Usage
///
/// ```
/// pub mod my_i18n {
///     shakehand::locale!("./test-locale/", fallback = "en");
/// }
/// ```
///
/// # Parameters
///
/// - `path` - i18n directory path (relative to `Cargo.toml`)
/// - `fallback` - optional, default language (defaults to `"en"`)
#[proc_macro]
pub fn locale(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as analyzer::ShakehandInput);

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

    let base_path = Path::new(&manifest_dir).join(&input.path);
    let base_path = just_fmt::fmt_path::fmt_path(&base_path).unwrap_or_else(|_| base_path.clone());

    let scanned_files = analyzer::scan_toml_files(&base_path);

    let mut parsed_files = Vec::new();
    let mut all_languages: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for (mod_path, file_path) in &scanned_files {
        if let Some(mut toml_file) = analyzer::parse_toml_file(file_path) {
            toml_file.module_path = mod_path.clone();
            all_languages.extend(toml_file.all_languages.clone());
            parsed_files.push(toml_file);
        }
    }

    // Read fallback chain from the crate's Cargo.toml [package.metadata.shakehand]
    let fallback_config = analyzer::read_fallback_from_manifest(&manifest_dir);

    // `default_fallback`: use `fallback.other` from Cargo.toml, else the macro's `fallback` param
    let default_fallback = fallback_config
        .as_ref()
        .map(|c| c.default_fallback.as_str())
        .unwrap_or(&input.fallback)
        .to_string();
    let fallback_map = fallback_config.map(|c| c.fallback_map).unwrap_or_default();

    // Add all languages referenced in the fallback chain (keys and values) to `all_languages`,
    // so the `Languages` enum includes variants that the chain may reference.
    for (k, v) in &fallback_map {
        all_languages.insert(k.clone());
        all_languages.insert(v.clone());
    }
    all_languages.insert(default_fallback.clone());

    if parsed_files.is_empty() {
        let lang_enum = shakehand::generate_module(
            parsed_files,
            all_languages,
            input.fallback,
            fallback_map,
            default_fallback,
            &input.path,
        );
        return TokenStream::from(quote::quote! {
            #lang_enum
        });
    }

    let generated = shakehand::generate_module(
        parsed_files,
        all_languages,
        input.fallback,
        fallback_map,
        default_fallback,
        &input.path,
    );
    TokenStream::from(generated)
}
