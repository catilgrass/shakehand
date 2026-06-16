use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use syn::{
    Ident, LitStr, Token,
    parse::{Parse, ParseStream},
};

/// Macro input: `shakehand::locale!("../i18n/", fallback = "en")`
pub struct ShakehandInput {
    pub path: String,
    pub fallback: String,
}

impl Parse for ShakehandInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path_lit: LitStr = input.parse()?;
        let path = path_lit.value();

        let mut fallback = String::from("en");

        if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            if input.peek(Ident) {
                let ident: Ident = input.parse()?;
                if ident == "fallback" {
                    let _: Token![=] = input.parse()?;
                    let fb: LitStr = input.parse()?;
                    fallback = fb.value();
                }
            }
        }

        Ok(ShakehandInput { path, fallback })
    }
}

/// A translation entry
#[derive(Clone)]
pub struct TranslationEntry {
    /// All keys
    pub key: String,

    /// All values
    pub values: BTreeMap<String, String>,

    /// Whether parameters exist, affects the function signature
    pub has_params: bool,

    /// Canonical parameter name list, valid when there is no conflict
    ///
    /// **Order**: taken from the first language that has parameters
    pub params: Vec<String>,

    /// Whether the entry has a conflict
    ///
    /// Conflicts arise in the following cases:
    /// - Inconsistent parameter counts
    /// - Inconsistent parameter names
    pub params_conflict: bool,
}

/// The parsing result of a toml file
#[derive(Clone)]
pub struct TomlFile {
    pub module_path: Vec<String>,
    pub struct_name: String,
    pub entries: Vec<TranslationEntry>,
    pub all_languages: BTreeSet<String>,
}

/// Convert a key to a valid Rust identifier
pub fn key_to_ident(key: &str) -> String {
    let snake = just_fmt::snake_case!(key);
    if snake.starts_with(|c: char| c.is_ascii_digit()) {
        format!("a{}", snake)
    } else if syn::parse_str::<Ident>(&snake).is_ok() {
        snake
    } else {
        format!("a_{}", snake)
    }
}

/// Generate a struct name (PascalCase) from a filename (without extension)
pub fn filename_to_struct_name(filename: &str) -> String {
    just_fmt::pascal_case!(filename)
}

/// Generate a module name (snake_case) from a path
pub fn path_to_mod_name(filename: &str) -> String {
    just_fmt::snake_case!(filename)
}

/// Convert a language key (e.g. `"en"`, `"zh_CN"`, `"en-US"`) to a valid enum variant name
/// Rules: `-` → `_`, preserve case, prepend `_` if starts with a digit
pub fn lang_to_variant(lang: &str) -> String {
    let raw = lang.replace('-', "_");
    if raw.starts_with(|c: char| c.is_ascii_digit()) {
        format!("_{}", raw)
    } else {
        raw
    }
}

/// Extract `%{param}` parameters from a string
pub fn extract_params(s: &str) -> Vec<String> {
    let mut params = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' && chars.peek() == Some(&'{') {
            chars.next();
            let mut param = String::new();
            while let Some(&next) = chars.peek() {
                if next == '}' {
                    chars.next();
                    break;
                }
                param.push(next);
                chars.next();
            }
            if !param.is_empty() && !params.contains(&param) {
                params.push(param);
            }
        }
    }
    params
}

/// Replace `%{param}` with `{}`, preserving occurrence order for `format!`
pub fn replace_params_with_format(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' && chars.peek() == Some(&'{') {
            chars.next();
            while let Some(&next) = chars.peek() {
                if next == '}' {
                    chars.next();
                    break;
                }
                chars.next();
            }
            result.push_str("{}");
        } else {
            result.push(c);
        }
    }
    result
}

/// Recursively scan a directory to collect all `.toml` files with their module paths
pub fn scan_toml_files(dir: &Path) -> Vec<(Vec<String>, PathBuf)> {
    let mut files = Vec::new();

    if !dir.exists() {
        return files;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let sub_files = scan_toml_files(&path);
                let dir_name = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                for (mut mod_path, file_path) in sub_files {
                    mod_path.insert(0, dir_name.clone());
                    files.push((mod_path, file_path));
                }
            } else if path.extension().is_some_and(|ext| ext == "toml") {
                files.push((vec![], path));
            }
        }
    }

    files
}

/// Parse a toml file
pub fn parse_toml_file(path: &Path) -> Option<TomlFile> {
    let content = fs::read_to_string(path).ok()?;
    let value: toml::Value = content.parse().ok()?;

    let table = value.as_table()?;

    let mut raw_entries: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut all_languages: BTreeSet<String> = BTreeSet::new();
    let mut all_keys: BTreeSet<String> = BTreeSet::new();

    for (lang_key, lang_val) in table {
        all_languages.insert(lang_key.clone());
        if let Some(lang_table) = lang_val.as_table() {
            for (entry_key, entry_val) in lang_table {
                all_keys.insert(entry_key.clone());
                if let Some(s) = entry_val.as_str() {
                    raw_entries
                        .entry(entry_key.clone())
                        .or_default()
                        .insert(lang_key.clone(), s.to_string());
                }
            }
        }
    }

    let mut entries = Vec::new();
    for key in &all_keys {
        let values = raw_entries.get(key).cloned().unwrap_or_default();

        // Collect parameter list for each language (in occurrence order)
        let lang_params: Vec<Vec<String>> = values.values().map(|v| extract_params(v)).collect();

        // Convert to parameter name sets for comparison (names only, not order)
        let param_sets: Vec<BTreeSet<String>> = lang_params
            .iter()
            .map(|p| p.iter().cloned().collect())
            .collect();

        // Conflict: inconsistent parameter name sets across languages (count or names differ)
        let params_conflict = if param_sets.is_empty() {
            false
        } else {
            let first = &param_sets[0];
            !param_sets.iter().skip(1).all(|p| p == first)
        };

        // Take parameter order from the first language that has parameters as canonical; empty params on conflict
        let (has_params, params) = if params_conflict {
            (false, vec![])
        } else {
            let canonical = values
                .values()
                .find_map(|v| {
                    let p = extract_params(v);
                    if !p.is_empty() { Some(p) } else { None }
                })
                .unwrap_or_default();
            (!canonical.is_empty(), canonical)
        };

        entries.push(TranslationEntry {
            key: key.clone(),
            values,
            has_params,
            params,
            params_conflict,
        });
    }

    let filename = path.file_stem()?.to_string_lossy().to_string();
    let struct_name = filename_to_struct_name(&filename);

    Some(TomlFile {
        module_path: vec![],
        struct_name,
        entries,
        all_languages,
    })
}
