# Sublime Color Scheme

[![Crates.io Version](https://img.shields.io/crates/v/sublime-color-scheme?style=plastic)](https://crates.io/crates/sublime-color-scheme)

This is a _simple_ library to parse a sublime color scheme to a [`syntect::highlighting::Theme`](https://docs.rs/syntect/latest/syntect/highlighting/struct.Theme.html).

I wrote this to use within my project [quellcode](https://github.com/Lepidopteran/quellcode), but other projects may find it useful.

## Supported features

### Colors

- [x] Hex
- [x] Hex with alpha
- [x] Named
- [x] RGB
- [x] RGBA
- [x] HSL
- [x] HSLA
- [x] HWB
- [x] HWB with alpha
- [x] Variables
- [X] Color Modifiers

### Adjusters

- [x] Lightness
- [x] Saturation
- [x] Alpha
- [x] Blend
- [x] Blend Alpha
- [x] Min Contrast

> Min Contrast is supported but I'm im not sure if I implemented correctly. Please open an issue if it doesn't work.

## TODO

- [ ] Foreground Adjust

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sublime-color-scheme = "0.1.2"
```

## Usage

```rust
    use sublime_color_scheme::ColorScheme;
    use syntect::highlighting::Theme;
    use std::{path::Path, fs::read_to_string};

    let path = Path::new("path/to/file.sublime-color-scheme");
    let scheme = read_to_string(path).expect("Failed to read file");
    let color_scheme = ColorScheme::from_str(&scheme).expect("Failed to parse theme");
    // or
    // let color_scheme = sublime_color_scheme::parse_color_scheme(&scheme).expect("Failed to parse theme");
    // or
    // let color_scheme = sublime_color_scheme::parse_color_scheme_file(path).expect("Failed to parse theme");

    Theme::try_from(scheme).expect("Failed to convert to theme");
```

## Contributing

Pull requests are welcome. 

Some things that need to be done is add more unit tests.

If you do decide to contribute, it is recommended to follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification when typing your commit messages, but it is not required.

## Credits

Thanks to [philpax](https://github.com/philpax) for the original implementation. 

See [here](https://github.com/trishume/syntect/issues/244#issuecomment-2480905939) for original

## License
[Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
