//! The mascot. The cow's face and mood track system load so the banner reacts
//! to what the machine is doing.

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
    /// Derived from the worst of CPU%, memory% and normalized load.
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

    /// Five lines of cow art; the eyes and sweat change with mood.
    pub fn art(self) -> [&'static str; 5] {
        match self {
            Mood::Grazing => [
                r"        ^__^",
                r"        (--)\_______",
                r"        (__)\       )\/\",
                r"            ||----w |",
                r"            ||     ||   ,,,",
            ],
            Mood::Content => [
                r"        ^__^",
                r"        (oo)\_______",
                r"        (__)\       )\/\",
                r"            ||----w |",
                r"            ||     ||",
            ],
            Mood::Busy => [
                r"        ^__^",
                r"        (oo)\_______",
                r"        (__)\       )\/\  ~",
                r"            ||----w |   ~",
                r"            ||     ||",
            ],
            Mood::Stressed => [
                r"     '   ^__^",
                r"      `  (Oo)\_______",
                r"        (__)\       )\/\  ~~",
                r"            ||----w |  ~~",
                r"            ||     ||",
            ],
            Mood::Panic => [
                r"   ' ' ' ^__^",
                r"    ` `  (@@)\_______",
                r"        (__)\       )\/\ !!",
                r"            ||----w |  !!",
                r"            ||     ||",
            ],
        }
    }
}
