use std::collections::HashMap;

use crate::{
    error::ParseError,
    parser::{parse_color, Adjuster, Color, ColorSpace},
};

use log::debug;
use palette::{
    color_difference::Wcag21RelativeContrast, Darken, Desaturate, Hsl, Hsla, Hwb, Hwba, IntoColor,
    Lighten, LinSrgba, Mix, Saturate, Srgba, WithAlpha,
};
use syntect::highlighting::Color as SyntectColor;

/// Parses a **Sublime Color Scheme** [color](https://www.sublimetext.com/docs/color_schemes.html#colors) and returns a [SyntectColor].
///
/// # Arguments
/// * `input` - The color to parse
/// * `variables` - The list of variables that were defined in the color scheme
///
/// # Returns
/// * [SyntectColor]
/// * [ParseError]
pub fn get_color(
    input: &str,
    variables: &HashMap<String, String>,
) -> Result<SyntectColor, ParseError> {
    let input = parse_color(input)?;
    let color = get_palette_color(input, variables)?;

    let sytent_color = SyntectColor {
        r: (color.red * 255.0) as u8,
        g: (color.green * 255.0) as u8,
        b: (color.blue * 225.0) as u8,
        a: (color.alpha * 255.0) as u8,
    };

    debug!("Got Color: {:?}", sytent_color);
    Ok(sytent_color)
}

/// Gets the [palette] crate color from [Color].
///
/// Always converts to [Srgba] since [SyntectColor] only supports rgb color space.
fn get_palette_color(
    input_color: Color,
    variables: &HashMap<String, String>,
) -> Result<Srgba, ParseError> {
    let color: Srgba<f32> = match input_color {
        Color::RGB(red, green, blue) => Srgba::new(
            red as f32 / 255.0,
            green as f32 / 255.0,
            blue as f32 / 255.0,
            1.0,
        ),
        Color::Hex(hex) => match hex.len() {
            3 => Srgba::new(
                u8::from_str_radix(&hex[0..1], 16)?,
                u8::from_str_radix(&hex[1..2], 16)?,
                u8::from_str_radix(&hex[2..3], 16)?,
                255,
            )
            .into(),
            6 => Srgba::new(
                u8::from_str_radix(&hex[0..2], 16)?,
                u8::from_str_radix(&hex[2..4], 16)?,
                u8::from_str_radix(&hex[4..6], 16)?,
                255,
            )
            .into(),
            _ => return Err(ParseError::InvalidHexColor),
        },
        Color::HexAlpha(hex) => match hex.len() {
            4 => Srgba::new(
                u8::from_str_radix(&hex[0..1], 16)?,
                u8::from_str_radix(&hex[1..2], 16)?,
                u8::from_str_radix(&hex[2..3], 16)?,
                u8::from_str_radix(&hex[3..4], 16)?,
            )
            .into(),
            8 => Srgba::new(
                u8::from_str_radix(&hex[0..2], 16)?,
                u8::from_str_radix(&hex[2..4], 16)?,
                u8::from_str_radix(&hex[4..6], 16)?,
                u8::from_str_radix(&hex[6..8], 16)?,
            )
            .into(),
            _ => return Err(ParseError::InvalidHexColor),
        },
        Color::RGBA(red, green, blue, alpha) => Srgba::new(
            red as f32 / 255.0,
            green as f32 / 255.0,
            blue as f32 / 255.0,
            alpha,
        ),
        Color::HSL(hue, saturation, lightness) => {
            Hsl::new(hue as f32, saturation, lightness).into_color()
        }
        Color::HSLA(hue, saturation, lightness, alpha) => {
            Hsla::new(hue as f32, saturation, lightness, alpha).into_color()
        }

        Color::HWB(hue, whiteness, blackness, alpha) => {
            if let Some(alpha) = alpha {
                Hwba::new(hue as f32, whiteness, blackness, alpha).into_color()
            } else {
                Hwb::new(hue as f32, whiteness, blackness).into_color()
            }
        }
        Color::Variable(name) => {
            let reference = variables.get(&name).ok_or(ParseError::UnknownVariable)?;
            let color = parse_color(reference)?;

            get_palette_color(color, variables)?
        }
        Color::Named(color) => Srgba::new(
            color.red as f32 / 255.0,
            color.green as f32 / 255.0,
            color.blue as f32 / 255.0,
            1.0,
        ),
        Color::Expression(color_type, adjusters) => {
            let mut current_color = get_palette_color(*color_type, variables)?;

            for adjuster in adjusters {
                match adjuster {
                    Adjuster::Alpha(alpha) => {
                        current_color = current_color.with_alpha(alpha);
                    }
                    Adjuster::Blend(color, percentage, color_space) => {
                        current_color = mix_colors(
                            current_color,
                            get_palette_color(color, variables)?,
                            percentage,
                            color_space,
                            true,
                        );
                    }
                    Adjuster::BlendAlpha(color, percentage, color_space) => {
                        current_color = mix_colors(
                            current_color,
                            get_palette_color(color, variables)?,
                            percentage,
                            color_space,
                            false,
                        );
                    }
                    Adjuster::Lightness(lightness, relative) => {
                        let mut hsl: Hsla = current_color.into_color();
                        let negative = lightness < 0.0;
                        if !relative {
                            hsl.lightness = lightness;
                            current_color = hsl.into_color();

                            continue;
                        }

                        if negative {
                            current_color = current_color.darken(-lightness);
                        } else {
                            current_color = current_color.lighten(lightness);
                        }
                    }
                    Adjuster::Saturation(saturation, relative) => {
                        let mut hsl: Hsla = current_color.into_color();

                        if !relative {
                            hsl.saturation = saturation;
                            current_color = hsl.into_color();

                            continue;
                        }

                        if saturation > 0.0 {
                            current_color = hsl.saturate(saturation).into_color();
                        } else {
                            current_color = hsl.desaturate(-saturation).into_color();
                        }
                    }
                    Adjuster::MinContrast(background_color, ratio) => {
                        let mut foreground: LinSrgba<f32> = current_color.into_linear();

                        let background: LinSrgba<f32> =
                            get_palette_color(background_color, variables)?.into_linear();

                        while foreground.relative_contrast(*background) < ratio {
                            let fg_luma = foreground.relative_luminance().luma;
                            let bg_luma = background.relative_luminance().luma;

                            if fg_luma > bg_luma {
                                foreground = foreground.lighten(0.1);
                            } else {
                                foreground = foreground.darken(0.1);
                            }

                            let new_luma = foreground.relative_luminance().luma;
                            if new_luma <= 0.0 || new_luma >= 1.0 {
                                break;
                            }
                        }

                        current_color = foreground.into_color();
                    }
                }
            }

            current_color
        }
    };

    debug!("Got Palette Color Color: {:?}", color);
    Ok(color)
}

/// Mixes two colors together.
///
/// It always returns the color in rgb space since that's what [SyntectColor] supports
///
/// # Arguments
/// * `base_color` - The base color to mix with
/// * `color` - The color to mix with the base color
/// * `percentage` - The percentage of the color to mix with the base color
/// * `color_space` - The color space to mix the colors in
/// * `preserve_base_alpha` - Whether to preserve the alpha of the base color
fn mix_colors(
    base_color: Srgba,
    color: Srgba,
    percentage: f32,
    color_space: Option<ColorSpace>,
    preserve_base_alpha: bool,
) -> Srgba {
    let mut base_color = base_color;
    let base_alpha = base_color.alpha;

    if let Some(color_space) = color_space {
        match color_space {
            ColorSpace::RGB => {
                base_color = base_color.mix(color, percentage);
            }
            ColorSpace::HSL => {
                let hsl_base: Hsl = base_color.into_color();
                let hsl_color: Hsl = color.into_color();

                base_color = hsl_base.mix(hsl_color, percentage).into_color();
            }
            ColorSpace::HWB => {
                let hwb_base: Hwb = base_color.into_color();
                let hwb_color: Hwb = color.into_color();

                base_color = hwb_base.mix(hwb_color, percentage).into_color();
            }
        }
    } else {
        base_color = base_color.mix(color, percentage);
    }

    if preserve_base_alpha {
        base_color = base_color.with_alpha(base_alpha);
    }

    base_color
}
