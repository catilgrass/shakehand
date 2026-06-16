# "Shake Hand!" 

> Let different languages shake hands with each other! 🤝

This is a **purely compile-time** Rust internationalization library. **All** localized strings are fully embedded into the binary at compile time, with only minimal runtime overhead.

# How to Use?

## 1. Add Dependency

Add the following to your `Cargo.toml`:

```toml
shakehand = "0.1"
```

## 2. Write Configuration Files

Create a directory under your project to serve as the root for translations, e.g. `./locale/`.
Create any `toml` file inside this directory, e.g. `./locale/global.toml`, representing a translation file.

In the file, group translations by language section. The key is the translation key, and the value is the translated text.
Text with parameters uses `%{parameter_name}` as placeholders, where the parameter name directly maps to the generated function's parameter name:

```toml
[en]
world = "world"
greeting = "Hello, %{someone}!"

[zh_CN]
world = "世界"
greeting = "你好，%{someone}！"
```

## 3. Load Translations

In your Rust code, use `shakehand::locale!` to hardcode the entire directory into a module:

```rust
pub mod translation {
    shakehand::locale!("./locale", fallback = "en");
}
```

The generated module will contain:
- A `Languages` enum listing all languages
- A `set_lang` function for switching the current language
- A unit struct `Global` (named after `global.toml`), whose associated functions are the translations for each key

## 4. Call Translations

Call translations just like ordinary functions, passing parameters by placeholder name:

```rust
use crate::translation::{Global, Languages, set_lang};

fn main() {
    set_lang(Languages::en);
    let greeting = Global::greeting("World");
    println!("{}", greeting);  // Hello, World!

    set_lang(Languages::zh_CN);
    let greeting = Global::greeting("世界");
    println!("{}", greeting);  // 你好，世界！
}
```

You can also pass the return value of another translation as a parameter, since each parameterless translation function returns a `&'static str`, completely allocation-free:

```rust
let greeting = Global::greeting(Global::world());
```



# Contributing

Directly open a PR to the [repository](https://github.com/catilgrass/shakehand) and mention [@Weicao-CatilGrass](https://github.com/Weicao-CatilGrass).



# License

MIT or Apache 2.0
