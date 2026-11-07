//! Monochrome, clean design system inspired by Familiar.

use eframe::egui::{Color32, Stroke};

#[allow(dead_code)]
pub struct Theme;

impl Theme {
    // Background layers (Pitch black to subtle elevated charcoal)
    pub const BG: Color32 = Color32::from_rgb(0, 0, 0);          // --bg: #000000
    pub const B1: Color32 = Color32::from_rgb(11, 11, 11);      // --b1: #0b0b0b
    pub const B2: Color32 = Color32::from_rgb(18, 18, 18);      // --b2: #121212
    pub const B3: Color32 = Color32::from_rgb(29, 29, 29);      // --b3: #1d1d1d (input/active pill)
    pub const B4: Color32 = Color32::from_rgb(38, 38, 38);      // --b4: #262626 (hover)

    // Foreground typography
    pub const FG: Color32 = Color32::from_rgb(242, 242, 242);   // --fg: #f2f2f2 (crisp text)
    pub const FG2: Color32 = Color32::from_rgb(209, 209, 214);  // --fg2: #d1d1d6
    pub const MUTE: Color32 = Color32::from_rgb(142, 142, 147); // --mute: #8e8e93
    pub const DIM: Color32 = Color32::from_rgb(85, 85, 90);     // --dim: #55555a

    // Monochrome Accent
    pub const ACCENT: Color32 = Color32::from_rgb(255, 255, 255); // --ac: #ffffff
    pub const ACCENT_FAINT: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 20);

    // Hairline borders
    pub const LINE: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 18); // --ln: 0.07 white
    pub const LINE_BRIGHT: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 46); // --ln-bright: 0.18 white

    // Status indicators
    pub const LOCK_GREEN: Color32 = Color32::from_rgb(52, 199, 89); // #34c759 (Apple/Familiar lock)
    pub const WARN_AMBER: Color32 = Color32::from_rgb(255, 149, 0); // #ff9500
    pub const STAR_YELLOW: Color32 = Color32::from_rgb(255, 214, 10); // #ffd60a

    // Border Strokes
    pub fn border_stroke() -> Stroke {
        Stroke::new(1.0, Self::LINE)
    }

    pub fn border_bright_stroke() -> Stroke {
        Stroke::new(1.0, Self::LINE_BRIGHT)
    }

    pub fn border_focus_stroke() -> Stroke {
        Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 75))
    }
}
