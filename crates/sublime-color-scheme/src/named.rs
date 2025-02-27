use std::{any, fmt::Display, str::FromStr};

use crate::error::ParseError;
pub type RGBColor = (u8, u8, u8);

/// An enum of every color keyword described in [W3C](https://www.w3.org/wiki/CSS/Properties/color/keywords).
///
/// Can convert to Hex String or [RGBColor]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    /// Black `#000000`
    Black,

    /// Silver `#C0C0C0`
    Silver,

    /// Gray `#808080`
    Gray,

    /// White `#FFFFFF`
    White,

    /// Maroon `#800000`
    Maroon,

    /// Red `#FF0000`
    Red,

    /// Purple `#800080`
    Purple,

    /// Fuchsia `#FF00FF`
    Fuchsia,

    /// Green `#008000`
    Green,

    /// Lime `#00FF00`
    Lime,

    /// Olive `#808000`
    Olive,

    /// Yellow `#FFFF00`
    Yellow,

    /// Navy `#000080`
    Navy,

    /// Blue `#0000FF`
    Blue,

    /// Teal `#008080`
    Teal,

    /// Aqua `#00FFFF`
    Aqua,

    /// Alice Blue `#f0f8ff`
    AliceBlue,

    /// Antique White `#faebd7`
    AntiqueWhite,

    /// Aquamarine `#7fffd4`
    Aquamarine,

    /// Azure `#f0ffff`
    Azure,

    /// Beige `#f5f5dc`
    Beige,

    /// Bisque `#ffe4c4`
    Bisque,

    /// Blanched Almond `#ffebcd`
    BlanchedAlmond,

    /// Blue Violet `#8a2be2`
    BlueViolet,

    /// Brown `#a52a2a`
    Brown,

    /// Burly Wood `#deb887`
    BurlyWood,

    /// Cadet Blue `#5f9ea0`
    CadetBlue,

    /// Chartreuse `#7fff00`
    Chartreuse,

    /// Chocolate `#d2691e`
    Chocolate,

    /// Coral `#ff7f50`
    Coral,

    /// Cornflower Blue `#6495ed`
    CornflowerBlue,

    /// Cornsilk `#fff8dc`
    Cornsilk,

    /// Crimson `#dc143c`
    Crimson,

    /// Cyan `#00ffff`
    Cyan,

    /// Dark Blue `#00008b`
    DarkBlue,

    /// Dark Cyan `#008b8b`
    DarkCyan,

    /// Dark Goldenrod `#b8860b`
    DarkGoldenrod,

    /// Dark Gray `#a9a9a9`
    DarkGray,

    /// Dark Green `#006400`
    DarkGreen,

    /// Dark Khaki `#bdb76b`
    DarkKhaki,

    /// Dark Magenta `#8b008b`
    DarkMagenta,

    /// Dark Olive Green `#556b2f`
    DarkOliveGreen,

    /// Dark Orange `#ff8c00`
    DarkOrange,

    /// Dark Orchid `#9932cc`
    DarkOrchid,

    /// Dark Red `#8b0000`
    DarkRed,

    /// Dark Salmon `#e9967a`
    DarkSalmon,

    /// Dark Sea Green `#8fbc8f`
    DarkSeaGreen,

    /// Dark Slate Blue `#483d8b`
    DarkSlateBlue,

    /// Dark Slate Gray `#2f4f4f`
    DarkSlateGray,

    /// Dark Turquoise `#00ced1`
    DarkTurquoise,

    /// Dark Violet `#9400d3`
    DarkViolet,

    /// Deep Pink `#ff1493`
    DeepPink,

    /// Deep Sky Blue `#00bfff`
    DeepSkyBlue,

    /// Dim Gray `#696969`
    DimGray,

    /// Dodger Blue `#1e90ff`
    DodgerBlue,

    /// Firebrick `#b22222`
    Firebrick,

    /// Floral White `#fffaf0`
    FloralWhite,

    /// Forest Green `#228b22`
    ForestGreen,

    /// Gainsboro `#dcdcdc`
    Gainsboro,

    /// Ghost White `#f8f8ff`
    GhostWhite,

    /// Gold `#ffd700`
    Gold,

    /// Goldenrod `#daa520`
    Goldenrod,

    /// Green Yellow `#adff2f`
    GreenYellow,

    /// Honeydew `#f0fff0`
    Honeydew,

    /// Hot Pink `#ff69b4`
    HotPink,

    /// Indian Red `#cd5c5c`
    IndianRed,

    /// Indigo `#4b0082`
    Indigo,

    /// Ivory `#fffff0`
    Ivory,

    /// Khaki `#f0e68c`
    Khaki,

    /// Lavender `#e6e6fa`
    Lavender,

    /// Lavender Blush `#fff0f5`
    LavenderBlush,

    /// Lawn Green `#7cfc00`
    LawnGreen,

    /// Lemon Chiffon `#fffacd`
    LemonChiffon,

    /// Light Blue `#add8e6`
    LightBlue,

    /// Light Coral `#f08080`
    LightCoral,

    /// Light Cyan `#e0ffff`
    LightCyan,

    /// Light Goldenrod Yellow `#fafad2`
    LightGoldenrodYellow,

    /// Light Gray `#d3d3d3`
    LightGray,

    /// Light Green `#90ee90`
    LightGreen,

    /// Light Pink `#ffb6c1`
    LightPink,

    /// Light Salmon `#ffa07a`
    LightSalmon,

    /// Light Sea Green `#20b2aa`
    LightSeaGreen,

    /// Light Sky Blue `#87cefa`
    LightSkyBlue,

    /// Light Slate Gray `#778899`
    LightSlateGray,

    /// Light Steel Blue `#b0c4de`
    LightSteelBlue,

    /// Light Yellow `#ffffe0`
    LightYellow,

    /// Lime Green `#32cd32`
    LimeGreen,

    /// Linen `#faf0e6`
    Linen,

    /// Magenta `#ff00ff`
    Magenta,

    /// Medium Aquamarine `#66cdaa`
    MediumAquamarine,

    /// Medium Blue `#0000cd`
    MediumBlue,

    /// Medium Orchid `#ba55d3`
    MediumOrchid,

    /// Medium Purple `#9370db`
    MediumPurple,

    /// Medium Sea Green `#3cb371`
    MediumSeaGreen,

    /// Medium Slate Blue `#7b68ee`
    MediumSlateBlue,

    /// Medium Spring Green `#00fa9a`
    MediumSpringGreen,

    /// Medium Turquoise `#48d1cc`
    MediumTurquoise,

    /// Medium Violet Red `#c71585`
    MediumVioletRed,

    /// Midnight Blue `#191970`
    MidnightBlue,

    /// Mint Cream `#f5fffa`
    MintCream,

    /// Misty Rose `#ffe4e1`
    MistyRose,

    /// Moccasin `#ffe4b5`
    Moccasin,

    /// Navajo White `#ffdead`
    NavajoWhite,

    /// Old Lace `#fdf5e6`
    OldLace,

    /// Olive Drab `#6b8e23`
    OliveDrab,

    /// Orange `#ffa500`
    Orange,

    /// Orange Red `#ff4500`
    OrangeRed,

    /// Orchid `#da70d6`
    Orchid,

    /// Pale Goldenrod `#eee8aa`
    PaleGoldenrod,

    /// Pale Green `#98fb98`
    PaleGreen,

    /// Pale Turquoise `#afeeee`
    PaleTurquoise,

    /// Pale Violet Red `#db7093`
    PaleVioletRed,

    /// Papaya Whip `#ffefd5`
    PapayaWhip,

    /// Peach Puff `#ffdab9`
    PeachPuff,

    /// Peru `#cd853f`
    Peru,

    /// Pink `#ffc0cb`
    Pink,

    /// Plum `#dda0dd`
    Plum,

    /// Powder Blue `#b0e0e6`
    PowderBlue,

    /// Rosy Brown `#bc8f8f`
    RosyBrown,

    /// Royal Blue `#4169e1`
    RoyalBlue,

    /// Saddle Brown `#8b4513`
    SaddleBrown,

    /// Salmon `#fa8072`
    Salmon,

    /// Sandy Brown `#f4a460`
    SandyBrown,

    /// Sea Green `#2e8b57`
    SeaGreen,

    /// Sea Shell `#fff5ee`
    SeaShell,

    /// Sienna `#a0522d`
    Sienna,

    /// Sky Blue `#87ceeb`
    SkyBlue,

    /// Slate Blue `#6a5acd`
    SlateBlue,

    /// Slate Gray `#708090`
    SlateGray,

    /// Snow `#fffafa`
    Snow,

    /// Spring Green `#00ff7f`
    SpringGreen,

    /// Steel Blue `#4682b4`
    SteelBlue,

    /// Tan `#d2b48c`
    Tan,

    /// Thistle `#d8bfd8`
    Thistle,

    /// Tomato `#ff6347`
    Tomato,

    /// Turquoise `#40e0d0`
    Turquoise,

    /// Violet `#ee82ee`
    Violet,

    /// Wheat `#f5deb3`
    Wheat,

    /// White Smoke `#f5f5f5`
    WhiteSmoke,

    /// Yellow Green `#9acd32`
    YellowGreen,
}

impl Keyword {
    /// Returns the RGB color of the keyword
    pub fn to_rgb(&self) -> RGBColor {
        match self {
            Keyword::Black => (0, 0, 0),
            Keyword::Silver => (192, 192, 192),
            Keyword::Gray => (128, 128, 128),
            Keyword::White => (255, 255, 255),
            Keyword::Maroon => (128, 0, 0),
            Keyword::Red => (255, 0, 0),
            Keyword::Purple => (128, 0, 128),
            Keyword::Fuchsia => (255, 0, 255),
            Keyword::Green => (0, 128, 0),
            Keyword::Lime => (0, 255, 0),
            Keyword::Olive => (128, 128, 0),
            Keyword::Yellow => (255, 255, 0),
            Keyword::Navy => (0, 0, 128),
            Keyword::Blue => (0, 0, 255),
            Keyword::Teal => (0, 128, 128),
            Keyword::Aqua => (0, 255, 255),
            Keyword::AliceBlue => (240, 248, 255),
            Keyword::AntiqueWhite => (250, 235, 215),
            Keyword::Aquamarine => (127, 255, 212),
            Keyword::Azure => (240, 255, 255),
            Keyword::Beige => (245, 245, 220),
            Keyword::Bisque => (255, 228, 196),
            Keyword::BlanchedAlmond => (255, 235, 205),
            Keyword::BlueViolet => (138, 43, 226),
            Keyword::Brown => (165, 42, 42),
            Keyword::BurlyWood => (222, 184, 135),
            Keyword::CadetBlue => (95, 158, 160),
            Keyword::Chartreuse => (127, 255, 0),
            Keyword::Chocolate => (210, 105, 30),
            Keyword::Coral => (255, 127, 80),
            Keyword::CornflowerBlue => (100, 149, 237),
            Keyword::Cornsilk => (255, 248, 220),
            Keyword::Crimson => (220, 20, 60),
            Keyword::Cyan => (0, 255, 255),
            Keyword::DarkBlue => (0, 0, 139),
            Keyword::DarkCyan => (0, 139, 139),
            Keyword::DarkGoldenrod => (184, 134, 11),
            Keyword::DarkGray => (169, 169, 169),
            Keyword::DarkGreen => (0, 100, 0),
            Keyword::DarkKhaki => (189, 183, 107),
            Keyword::DarkMagenta => (139, 0, 139),
            Keyword::DarkOliveGreen => (85, 107, 47),
            Keyword::DarkOrange => (255, 140, 0),
            Keyword::DarkOrchid => (153, 50, 204),
            Keyword::DarkRed => (139, 0, 0),
            Keyword::DarkSalmon => (233, 150, 122),
            Keyword::DarkSeaGreen => (143, 188, 143),
            Keyword::DarkSlateBlue => (72, 61, 139),
            Keyword::DarkSlateGray => (47, 79, 79),
            Keyword::DarkTurquoise => (0, 206, 209),
            Keyword::DarkViolet => (148, 0, 211),
            Keyword::DeepPink => (255, 20, 147),
            Keyword::DeepSkyBlue => (0, 191, 255),
            Keyword::DimGray => (105, 105, 105),
            Keyword::DodgerBlue => (30, 144, 255),
            Keyword::Firebrick => (178, 34, 34),
            Keyword::FloralWhite => (255, 250, 240),
            Keyword::ForestGreen => (34, 139, 34),
            Keyword::Gainsboro => (220, 220, 220),
            Keyword::GhostWhite => (248, 248, 255),
            Keyword::Gold => (255, 215, 0),
            Keyword::Goldenrod => (218, 165, 32),
            Keyword::GreenYellow => (173, 255, 47),
            Keyword::Honeydew => (240, 255, 240),
            Keyword::HotPink => (255, 105, 180),
            Keyword::IndianRed => (205, 92, 92),
            Keyword::Indigo => (75, 0, 130),
            Keyword::Ivory => (255, 255, 240),
            Keyword::Khaki => (240, 230, 140),
            Keyword::Lavender => (230, 230, 250),
            Keyword::LavenderBlush => (255, 240, 245),
            Keyword::LawnGreen => (124, 252, 0),
            Keyword::LemonChiffon => (255, 250, 205),
            Keyword::LightBlue => (173, 216, 230),
            Keyword::LightCoral => (240, 128, 128),
            Keyword::LightCyan => (224, 255, 255),
            Keyword::LightGoldenrodYellow => (250, 250, 210),
            Keyword::LightGray => (211, 211, 211),
            Keyword::LightGreen => (144, 238, 144),
            Keyword::LightPink => (255, 182, 193),
            Keyword::LightSalmon => (255, 160, 122),
            Keyword::LightSeaGreen => (32, 178, 170),
            Keyword::LightSkyBlue => (135, 206, 250),
            Keyword::LightSlateGray => (119, 136, 153),
            Keyword::LightSteelBlue => (176, 196, 222),
            Keyword::LightYellow => (255, 255, 224),
            Keyword::LimeGreen => (50, 205, 50),
            Keyword::Linen => (250, 240, 230),
            Keyword::Magenta => (255, 0, 255),
            Keyword::MediumAquamarine => (102, 205, 170),
            Keyword::MediumBlue => (0, 0, 205),
            Keyword::MediumOrchid => (186, 85, 211),
            Keyword::MediumPurple => (147, 112, 219),
            Keyword::MediumSeaGreen => (60, 179, 113),
            Keyword::MediumSlateBlue => (123, 104, 238),
            Keyword::MediumSpringGreen => (0, 250, 154),
            Keyword::MediumTurquoise => (72, 209, 204),
            Keyword::MediumVioletRed => (199, 21, 133),
            Keyword::MidnightBlue => (25, 25, 112),
            Keyword::MintCream => (245, 255, 250),
            Keyword::MistyRose => (255, 228, 225),
            Keyword::Moccasin => (255, 228, 181),
            Keyword::NavajoWhite => (255, 222, 173),
            Keyword::OldLace => (253, 245, 230),
            Keyword::OliveDrab => (107, 142, 35),
            Keyword::Orange => (255, 165, 0),
            Keyword::OrangeRed => (255, 69, 0),
            Keyword::Orchid => (218, 112, 214),
            Keyword::PaleGoldenrod => (238, 232, 170),
            Keyword::PaleGreen => (152, 251, 152),
            Keyword::PaleTurquoise => (175, 238, 238),
            Keyword::PaleVioletRed => (219, 112, 147),
            Keyword::PapayaWhip => (255, 239, 213),
            Keyword::PeachPuff => (255, 218, 185),
            Keyword::Peru => (205, 133, 63),
            Keyword::Pink => (255, 192, 203),
            Keyword::Plum => (221, 160, 221),
            Keyword::PowderBlue => (176, 224, 230),
            Keyword::RosyBrown => (188, 143, 143),
            Keyword::RoyalBlue => (65, 105, 225),
            Keyword::SaddleBrown => (139, 69, 19),
            Keyword::Salmon => (250, 128, 114),
            Keyword::SandyBrown => (244, 164, 96),
            Keyword::SeaGreen => (46, 139, 87),
            Keyword::SeaShell => (255, 245, 238),
            Keyword::Sienna => (160, 82, 45),
            Keyword::SkyBlue => (135, 206, 235),
            Keyword::SlateBlue => (106, 90, 205),
            Keyword::SlateGray => (112, 128, 144),
            Keyword::Snow => (255, 250, 250),
            Keyword::SpringGreen => (0, 255, 127),
            Keyword::SteelBlue => (70, 130, 180),
            Keyword::Tan => (210, 180, 140),
            Keyword::Thistle => (216, 191, 216),
            Keyword::Tomato => (255, 99, 71),
            Keyword::Turquoise => (64, 224, 208),
            Keyword::Violet => (238, 130, 238),
            Keyword::Wheat => (245, 222, 179),
            Keyword::WhiteSmoke => (245, 245, 245),
            Keyword::YellowGreen => (154, 205, 50),
        }
    }

    fn to_hex(&self) -> String {
        let (r, g, b) = self.to_rgb();
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}

impl FromStr for Keyword {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "black" => Ok(Keyword::Black),
            "silver" => Ok(Keyword::Silver),
            "gray" => Ok(Keyword::Gray),
            "white" => Ok(Keyword::White),
            "maroon" => Ok(Keyword::Maroon),
            "red" => Ok(Keyword::Red),
            "purple" => Ok(Keyword::Purple),
            "fuchsia" => Ok(Keyword::Fuchsia),
            "green" => Ok(Keyword::Green),
            "lime" => Ok(Keyword::Lime),
            "olive" => Ok(Keyword::Olive),
            "yellow" => Ok(Keyword::Yellow),
            "navy" => Ok(Keyword::Navy),
            "blue" => Ok(Keyword::Blue),
            "teal" => Ok(Keyword::Teal),
            "aqua" => Ok(Keyword::Aqua),
            "aliceblue" => Ok(Keyword::AliceBlue),
            "antiquewhite" => Ok(Keyword::AntiqueWhite),
            "aquamarine" => Ok(Keyword::Aquamarine),
            "azure" => Ok(Keyword::Azure),
            "beige" => Ok(Keyword::Beige),
            "bisque" => Ok(Keyword::Bisque),
            "blanchedalmond" => Ok(Keyword::BlanchedAlmond),
            "blueviolet" => Ok(Keyword::BlueViolet),
            "brown" => Ok(Keyword::Brown),
            "burlywood" => Ok(Keyword::BurlyWood),
            "cadetblue" => Ok(Keyword::CadetBlue),
            "chartreuse" => Ok(Keyword::Chartreuse),
            "chocolate" => Ok(Keyword::Chocolate),
            "coral" => Ok(Keyword::Coral),
            "cornflowerblue" => Ok(Keyword::CornflowerBlue),
            "cornsilk" => Ok(Keyword::Cornsilk),
            "crimson" => Ok(Keyword::Crimson),
            "cyan" => Ok(Keyword::Cyan),
            "darkblue" => Ok(Keyword::DarkBlue),
            "darkcyan" => Ok(Keyword::DarkCyan),
            "darkgoldenrod" => Ok(Keyword::DarkGoldenrod),
            "darkgray" => Ok(Keyword::DarkGray),
            "darkgreen" => Ok(Keyword::DarkGreen),
            "darkkhaki" => Ok(Keyword::DarkKhaki),
            "darkmagenta" => Ok(Keyword::DarkMagenta),
            "darkolivegreen" => Ok(Keyword::DarkOliveGreen),
            "darkorange" => Ok(Keyword::DarkOrange),
            "darkorchid" => Ok(Keyword::DarkOrchid),
            "darkred" => Ok(Keyword::DarkRed),
            "darksalmon" => Ok(Keyword::DarkSalmon),
            "darkseagreen" => Ok(Keyword::DarkSeaGreen),
            "darkslateblue" => Ok(Keyword::DarkSlateBlue),
            "darkslategray" => Ok(Keyword::DarkSlateGray),
            "darkturquoise" => Ok(Keyword::DarkTurquoise),
            "darkviolet" => Ok(Keyword::DarkViolet),
            "deeppink" => Ok(Keyword::DeepPink),
            "deepskyblue" => Ok(Keyword::DeepSkyBlue),
            "dimgray" => Ok(Keyword::DimGray),
            "dodgerblue" => Ok(Keyword::DodgerBlue),
            "firebrick" => Ok(Keyword::Firebrick),
            "floralwhite" => Ok(Keyword::FloralWhite),
            "forestgreen" => Ok(Keyword::ForestGreen),
            "gainsboro" => Ok(Keyword::Gainsboro),
            "ghostwhite" => Ok(Keyword::GhostWhite),
            "gold" => Ok(Keyword::Gold),
            "goldenrod" => Ok(Keyword::Goldenrod),
            "greenyellow" => Ok(Keyword::GreenYellow),
            "honeydew" => Ok(Keyword::Honeydew),
            "hotpink" => Ok(Keyword::HotPink),
            "indianred" => Ok(Keyword::IndianRed),
            "indigo" => Ok(Keyword::Indigo),
            "ivory" => Ok(Keyword::Ivory),
            "khaki" => Ok(Keyword::Khaki),
            "lavender" => Ok(Keyword::Lavender),
            "lavenderblush" => Ok(Keyword::LavenderBlush),
            "lawngreen" => Ok(Keyword::LawnGreen),
            "lemonchiffon" => Ok(Keyword::LemonChiffon),
            "lightblue" => Ok(Keyword::LightBlue),
            "lightcoral" => Ok(Keyword::LightCoral),
            "lightcyan" => Ok(Keyword::LightCyan),
            "lightgoldenrodyellow" => Ok(Keyword::LightGoldenrodYellow),
            "lightgray" => Ok(Keyword::LightGray),
            "lightgreen" => Ok(Keyword::LightGreen),
            "lightpink" => Ok(Keyword::LightPink),
            "lightsalmon" => Ok(Keyword::LightSalmon),
            "lightseagreen" => Ok(Keyword::LightSeaGreen),
            "lightskyblue" => Ok(Keyword::LightSkyBlue),
            "lightslategray" => Ok(Keyword::LightSlateGray),
            "lightsteelblue" => Ok(Keyword::LightSteelBlue),
            "lightyellow" => Ok(Keyword::LightYellow),
            "limegreen" => Ok(Keyword::LimeGreen),
            "linen" => Ok(Keyword::Linen),
            "magenta" => Ok(Keyword::Magenta),
            "mediumaquamarine" => Ok(Keyword::MediumAquamarine),
            "mediumblue" => Ok(Keyword::MediumBlue),
            "mediumorchid" => Ok(Keyword::MediumOrchid),
            "mediumpurple" => Ok(Keyword::MediumPurple),
            "mediumseagreen" => Ok(Keyword::MediumSeaGreen),
            "mediumslateblue" => Ok(Keyword::MediumSlateBlue),
            "mediumspringgreen" => Ok(Keyword::MediumSpringGreen),
            "mediumturquoise" => Ok(Keyword::MediumTurquoise),
            "mediumvioletred" => Ok(Keyword::MediumVioletRed),
            "midnightblue" => Ok(Keyword::MidnightBlue),
            "mintcream" => Ok(Keyword::MintCream),
            "mistyrose" => Ok(Keyword::MistyRose),
            "moccasin" => Ok(Keyword::Moccasin),
            "navajowhite" => Ok(Keyword::NavajoWhite),
            "oldlace" => Ok(Keyword::OldLace),
            "olivedrab" => Ok(Keyword::OliveDrab),
            "orange" => Ok(Keyword::Orange),
            "orangered" => Ok(Keyword::OrangeRed),
            "orchid" => Ok(Keyword::Orchid),
            "palegoldenrod" => Ok(Keyword::PaleGoldenrod),
            "palegreen" => Ok(Keyword::PaleGreen),
            "paleturquoise" => Ok(Keyword::PaleTurquoise),
            "palevioletred" => Ok(Keyword::PaleVioletRed),
            "papayawhip" => Ok(Keyword::PapayaWhip),
            "peachpuff" => Ok(Keyword::PeachPuff),
            "peru" => Ok(Keyword::Peru),
            "pink" => Ok(Keyword::Pink),
            "plum" => Ok(Keyword::Plum),
            "powderblue" => Ok(Keyword::PowderBlue),
            "rosybrown" => Ok(Keyword::RosyBrown),
            "royalblue" => Ok(Keyword::RoyalBlue),
            "saddlebrown" => Ok(Keyword::SaddleBrown),
            "salmon" => Ok(Keyword::Salmon),
            "sandybrown" => Ok(Keyword::SandyBrown),
            "seagreen" => Ok(Keyword::SeaGreen),
            "seashell" => Ok(Keyword::SeaShell),
            "sienna" => Ok(Keyword::Sienna),
            "skyblue" => Ok(Keyword::SkyBlue),
            "slateblue" => Ok(Keyword::SlateBlue),
            "slategray" => Ok(Keyword::SlateGray),
            "snow" => Ok(Keyword::Snow),
            "springgreen" => Ok(Keyword::SpringGreen),
            "steelblue" => Ok(Keyword::SteelBlue),
            "tan" => Ok(Keyword::Tan),
            "thistle" => Ok(Keyword::Thistle),
            "tomato" => Ok(Keyword::Tomato),
            "turquoise" => Ok(Keyword::Turquoise),
            "violet" => Ok(Keyword::Violet),
            "wheat" => Ok(Keyword::Wheat),
            "whitesmoke" => Ok(Keyword::WhiteSmoke),
            "yellowgreen" => Ok(Keyword::YellowGreen),
            _ => Err(ParseError::InvalidKeyword),
        }
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            any::type_name_of_val(self).split("::").last().unwrap()
        )
    }
}

impl From<Keyword> for String {
    fn from(k: Keyword) -> String {
        k.to_string()
    }
}
