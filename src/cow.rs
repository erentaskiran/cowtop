//! The mascot. The cow's face and mood track system load.
//! Includes idle animation frames for variety.

use ratatui::style::Color;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Grazing,
    Content,
    Busy,
    Stressed,
    Panic,
}

impl Mood {
    pub fn from_load(cpu_percent: f64, mem_percent: f64, load1: f64, cores: usize) -> Mood {
        let load_pct = if cores > 0 {
            100.0 * load1 / cores as f64
        } else {
            0.0
        };
        let level = cpu_percent.max(mem_percent).max(load_pct);
        if level >= 90.0 {
            Mood::Panic
        } else if level >= 75.0 {
            Mood::Stressed
        } else if level >= 50.0 {
            Mood::Busy
        } else if level >= 25.0 {
            Mood::Content
        } else {
            Mood::Grazing
        }
    }

    pub fn color(self) -> Color {
        match self {
            Mood::Grazing => Color::Green,
            Mood::Content => Color::LightGreen,
            Mood::Busy => Color::Yellow,
            Mood::Stressed => Color::LightRed,
            Mood::Panic => Color::Red,
        }
    }

    pub fn badge(self) -> &'static str {
        match self {
            Mood::Grazing => "GRAZING",
            Mood::Content => "CONTENT",
            Mood::Busy => "BUSY",
            Mood::Stressed => "STRESSED",
            Mood::Panic => "PANIC",
        }
    }

    pub fn phrase(self) -> &'static str {
        match self {
            Mood::Grazing => "moo~ just chewing some cud",
            Mood::Content => "mooo, all is well in the pasture",
            Mood::Busy => "moo! plenty of grass to munch",
            Mood::Stressed => "MOOO?! the field is getting full",
            Mood::Panic => "MOOOO!! send hay, send help!!",
        }
    }

    /// Five lines of cow art. The eyes and sweat change with mood.
    /// `frame` picks an idle animation variant.
    pub fn art(self, frame: u64) -> [&'static str; 5] {
        let tick = (frame / 30) % 3;
        match self {
            Mood::Grazing => match tick {
                0 => [
                    r"        ^__^",
                    r"        (--)\_______",
                    r"        (__)\       )\/\",
                    r"            ||----w |",
                    r"            ||     ||   ,,,",
                ],
                1 => [
                    r"        ^__^",
                    r"        (--)\_______",
                    r"        (__)\       )\/\",
                    r"            ||----w |   ,",
                    r"            ||     ||  ,,,",
                ],
                _ => [
                    r"        ^__^",
                    r"        (oo)\_______",
                    r"        (__)\       )\/\",
                    r"            ||----w |",
                    r"            ||     ||   ,,,",
                ],
            },
            Mood::Content => [
                r"        ^__^",
                r"        (oo)\_______",
                r"        (__)\       )\/\",
                r"            ||----w |",
                r"            ||     ||",
            ],
            Mood::Busy => match tick {
                0 => [
                    r"        ^__^",
                    r"        (oo)\_______",
                    r"        (__)\       )\/\  ~",
                    r"            ||----w |   ~",
                    r"            ||     ||",
                ],
                _ => [
                    r"        ^__^",
                    r"        (oO)\_______",
                    r"        (__)\       )\/\  ~",
                    r"            ||----w |   ~~",
                    r"            ||     ||",
                ],
            },
            Mood::Stressed => match tick {
                0 => [
                    r"     '   ^__^",
                    r"      `  (Oo)\_______",
                    r"        (__)\       )\/\  ~~",
                    r"            ||----w |  ~~",
                    r"            ||     ||",
                ],
                _ => [
                    r"    ' '  ^__^",
                    r"     ` ` (oO)\_______",
                    r"        (__)\       )\/\ ~~~",
                    r"            ||----w | ~~~",
                    r"            ||     ||",
                ],
            },
            Mood::Panic => match tick {
                0 => [
                    r"   ' ' ' ^__^",
                    r"    ` `  (@@)\_______",
                    r"        (__)\       )\/\ !!",
                    r"            ||----w |  !!",
                    r"            ||     ||",
                ],
                _ => [
                    r"  ' ' '  ^__^",
                    r"   ` ` ` (Xx)\_______",
                    r"        (__)\       )\/\ !!!",
                    r"            ||----w | !!!",
                    r"            ||     ||",
                ],
            },
        }
    }

}
