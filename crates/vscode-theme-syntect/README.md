# VSCode Theme Syntect

[![Crates.io Version](https://img.shields.io/crates/v/sublime-color-scheme?style=plastic)](https://crates.io/crates/syntect-vscode)

This is a simple library to parse a vscode theme to a [`syntect::highlighting::Theme`](https://docs.rs/syntect/latest/syntect/highlighting/struct.Theme.html).

I wrote this to use within my project [quellcode](https://github.com/Lepidopteran/quellcode), but other projects may find it useful.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
syntect-vscode = "0.1.0"
```

## Usage

Parse from a string using `parse_vscode_theme`:

```rust
use syntect::highlighting::Theme;
use vscode_theme_syntect::parse_vscode_theme;

let theme = parse_vscode_theme(include_str!("../assets/palenight.json")).expect("Failed to parse theme");

assert!(Theme::try_from(theme).is_ok());
```

Parse from a string using `VscodeTheme::from_str`:

```rust
use syntect::highlighting::Theme;
use vscode_theme_syntect::VscodeTheme;
use std::str::FromStr;

let theme = VscodeTheme::from_str(include_str!("../assets/palenight.json")).expect("Failed to parse theme");

assert!(Theme::try_from(theme).is_ok());
```

Parse from a file using `parse_vscode_theme_from_file`

 ```rust
use std::path::Path;
use syntect::highlighting::Theme;
use vscode_theme_syntect::parse_vscode_theme_from_file;

let theme = parse_vscode_theme_from_file(Path::new("assets/palenight.json")).expect("Failed to parse theme");

assert!(Theme::try_from(theme).is_ok());
```

## Contributing

Pull requests are welcome. 

Some things that need to be done is add more unit tests.

If you do decide to contribute, it is recommended to follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification when typing your commit messages.

## License
[Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
