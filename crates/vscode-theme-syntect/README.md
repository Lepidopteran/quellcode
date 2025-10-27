# Syntect VSCode

[![Crates.io Version](https://img.shields.io/crates/v/sublime-color-scheme?style=plastic)](https://crates.io/crates/syntect-vscode)

This is a simple library to parse a vscode theme to a [`syntect::highlighting::Theme`](https://docs.rs/syntect/latest/syntect/highlighting/struct.Theme.html).

I wrote this to use within my project [quellcode](https://github.com/Lepidopteran/quellcode), but other projects may find it useful.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
syntect-vscode = "0.1.0"
```

## Contributing

Pull requests are welcome. 

Some things that need to be done is add more unit tests.

If you do decide to contribute, it is recommended to follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification when typing your commit messages, but it is not required.

## License
[Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
