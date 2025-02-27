
// TODO: Add min contrast
#[derive(Debug, PartialEq)]
pub enum Adjuster {
    /// Blend two colors `(Color, Percentage, ColorSpace)`
    Blend(Color, f32, Option<ColorSpace>),
    /// Blend two colors with alpha `(Color, Percentage, ColorSpace)`
    BlendAlpha(Color, f32, Option<ColorSpace>),
    /// Adjust the opacity of the color `(Percentage)`
    Alpha(f32),
    /// Adjust the hue of the color `(Percentage)`
    Saturation(f32),
    /// Adjust the saturation of the color `(Percentage)`
    Lightness(f32),
}

// TODO: Add min contrast
#[derive(Debug, PartialEq)]
pub enum AdjusterKind {
    Blend,
    BlendAlpha,
    Alpha,
    Saturation,
    Lightness,
} 
