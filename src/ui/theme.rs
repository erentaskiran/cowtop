use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType},
};

/// Complete color theme for the cowtop TUI.
#[derive(Clone, Debug)]
pub struct Theme {
    pub name: &'static str,
    // Primary palette
    pub meadow: Color,     // main green / brand
    pub clover: Color,     // dark green
    pub grass: Color,      // light green
    pub milk: Color,       // off-white text
    pub cream: Color,      // warm body text
    pub daisy: Color,      // accent yellow/gold
    pub blossom: Color,    // pink accent
    pub sky: Color,        // blue
    pub earth: Color,      // brown
    pub spot: Color,       // near-black background
    pub dim: Color,        // muted/dim text
    // Semantic
    pub bg: Color,
    pub danger: Color,
    pub warning: Color,
    pub success: Color,
    // Block borders
    pub block_border: Color,
    pub block_title: Color,
    // Gauge backgrounds
    pub gauge_cpu_bg: Color,
    pub gauge_mem_bg: Color,
    pub gauge_disk_bg: Color,
}

impl Theme {
    pub fn gauge_color(&self, pct: f64) -> Color {
        if pct >= 85.0 {
            self.danger
        } else if pct >= 60.0 {
            self.warning
        } else {
            self.success
        }
    }

    pub fn temp_color(&self, c: f64) -> Color {
        if c >= 90.0 { self.danger }
        else if c >= 70.0 { self.warning }
        else if c >= 40.0 { self.meadow }
        else { self.sky }
    }

    pub fn conn_color(&self, state: &str) -> Color {
        match state {
            "ESTAB" => self.success,
            "LISTEN" => self.sky,
            "TIME_WAIT" | "CLOSE_WAIT" | "CLOSING" => self.warning,
            _ => self.cream,
        }
    }

    pub fn pasture_block<'a>(&self, title: &str) -> Block<'a> {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.block_border))
            .title(Span::styled(
                format!(" ✿ {} ✿ ", title),
                Style::default().fg(self.daisy).add_modifier(Modifier::BOLD),
            ))
            .title_alignment(Alignment::Left)
    }

    pub fn cpu_block<'a>(&self, title: &str) -> Block<'a> {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.meadow))
            .title(Span::styled(
                format!(" ≋ {} ≋ ", title),
                Style::default().fg(self.grass).add_modifier(Modifier::BOLD),
            ))
            .title_bottom(Span::styled(
                " ·~·~·~·~·~·~·~·~· ",
                Style::default().fg(self.clover),
            ))
    }

    pub fn mem_block<'a>(&self, title: &str) -> Block<'a> {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.sky))
            .title(Span::styled(
                format!(" ✿ {} ✿ ", title),
                Style::default().fg(self.milk).add_modifier(Modifier::BOLD),
            ))
            .title_bottom(Span::styled(
                " ·≋·≋·≋·≋·≋·≋·≋·≋· ",
                Style::default().fg(Color::Rgb(60, 150, 195)),
            ))
    }

    pub fn net_block<'a>(&self, title: &str) -> Block<'a> {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(
                format!(" ⊹ {} ⊹ ", title),
                Style::default().fg(self.sky).add_modifier(Modifier::BOLD),
            ))
            .title_bottom(Span::styled(
                " ·⊹·⊹·⊹·⊹·⊹·⊹·⊹·⊹· ",
                Style::default().fg(Color::Rgb(50, 160, 185)),
            ))
    }

    pub fn disk_block<'a>(&self, title: &str) -> Block<'a> {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(180, 130, 60)))
            .title(Span::styled(
                format!(" ⊕ {} ⊕ ", title),
                Style::default().fg(self.daisy).add_modifier(Modifier::BOLD),
            ))
            .title_bottom(Span::styled(
                " ·⊕·⊕·⊕·⊕·⊕·⊕·⊕·⊕· ",
                Style::default().fg(self.earth),
            ))
    }

    pub fn proc_block<'a>(&self, title: &str) -> Block<'a> {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.blossom))
            .title(Span::styled(
                format!(" ※ {} ※ ", title),
                Style::default().fg(self.milk).add_modifier(Modifier::BOLD),
            ))
            .title_bottom(Span::styled(
                " ·※·※·※·※·※·※·※·※· ",
                Style::default().fg(Color::Rgb(210, 90, 130)),
            ))
    }

    pub fn sensor_block<'a>(&self, title: &str) -> Block<'a> {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.warning))
            .title(Span::styled(
                format!(" ⚡ {} ⚡ ", title),
                Style::default().fg(self.daisy).add_modifier(Modifier::BOLD),
            ))
    }
}

// ── Theme definitions ──────────────────────────────────────────────

pub const PASTURE: Theme = Theme {
    name: "Pasture",
    meadow: Color::Rgb(72, 158, 52),
    clover: Color::Rgb(45, 120, 38),
    grass: Color::Rgb(120, 195, 85),
    milk: Color::Rgb(248, 244, 234),
    cream: Color::Rgb(230, 220, 195),
    daisy: Color::Rgb(255, 210, 50),
    blossom: Color::Rgb(255, 130, 165),
    sky: Color::Rgb(90, 185, 225),
    earth: Color::Rgb(140, 95, 50),
    spot: Color::Rgb(35, 32, 28),
    dim: Color::Rgb(108, 122, 103),
    bg: Color::Rgb(24, 22, 18),
    danger: Color::Red,
    warning: Color::Yellow,
    success: Color::Rgb(72, 158, 52),
    block_border: Color::Rgb(72, 158, 52),
    block_title: Color::Rgb(255, 210, 50),
    gauge_cpu_bg: Color::Rgb(28, 38, 25),
    gauge_mem_bg: Color::Rgb(25, 35, 45),
    gauge_disk_bg: Color::Rgb(38, 28, 18),
};

pub const MIDNIGHT: Theme = Theme {
    name: "Midnight",
    meadow: Color::Rgb(100, 140, 255),
    clover: Color::Rgb(60, 90, 180),
    grass: Color::Rgb(140, 170, 255),
    milk: Color::Rgb(220, 225, 240),
    cream: Color::Rgb(180, 190, 210),
    daisy: Color::Rgb(255, 220, 100),
    blossom: Color::Rgb(255, 150, 180),
    sky: Color::Rgb(120, 200, 255),
    earth: Color::Rgb(160, 130, 90),
    spot: Color::Rgb(16, 18, 30),
    dim: Color::Rgb(90, 100, 130),
    bg: Color::Rgb(10, 12, 22),
    danger: Color::Rgb(255, 80, 80),
    warning: Color::Rgb(255, 200, 60),
    success: Color::Rgb(80, 200, 120),
    block_border: Color::Rgb(100, 140, 255),
    block_title: Color::Rgb(255, 220, 100),
    gauge_cpu_bg: Color::Rgb(18, 20, 35),
    gauge_mem_bg: Color::Rgb(18, 20, 35),
    gauge_disk_bg: Color::Rgb(18, 20, 35),
};

pub const SUNSET: Theme = Theme {
    name: "Sunset",
    meadow: Color::Rgb(230, 120, 50),
    clover: Color::Rgb(180, 80, 30),
    grass: Color::Rgb(255, 160, 80),
    milk: Color::Rgb(255, 240, 225),
    cream: Color::Rgb(240, 210, 180),
    daisy: Color::Rgb(255, 230, 100),
    blossom: Color::Rgb(255, 130, 130),
    sky: Color::Rgb(130, 170, 255),
    earth: Color::Rgb(170, 110, 60),
    spot: Color::Rgb(32, 18, 24),
    dim: Color::Rgb(155, 110, 85),
    bg: Color::Rgb(24, 12, 16),
    danger: Color::Rgb(255, 60, 60),
    warning: Color::Rgb(255, 180, 40),
    success: Color::Rgb(200, 140, 60),
    block_border: Color::Rgb(230, 120, 50),
    block_title: Color::Rgb(255, 230, 100),
    gauge_cpu_bg: Color::Rgb(35, 18, 22),
    gauge_mem_bg: Color::Rgb(28, 18, 28),
    gauge_disk_bg: Color::Rgb(35, 22, 16),
};

pub const FOREST: Theme = Theme {
    name: "Forest",
    meadow: Color::Rgb(50, 180, 80),
    clover: Color::Rgb(20, 100, 30),
    grass: Color::Rgb(80, 210, 100),
    milk: Color::Rgb(225, 240, 220),
    cream: Color::Rgb(190, 215, 180),
    daisy: Color::Rgb(220, 240, 80),
    blossom: Color::Rgb(220, 120, 160),
    sky: Color::Rgb(80, 190, 200),
    earth: Color::Rgb(120, 80, 40),
    spot: Color::Rgb(18, 28, 16),
    dim: Color::Rgb(90, 120, 85),
    bg: Color::Rgb(10, 20, 8),
    danger: Color::Rgb(255, 70, 70),
    warning: Color::Rgb(220, 200, 40),
    success: Color::Rgb(50, 200, 80),
    block_border: Color::Rgb(50, 180, 80),
    block_title: Color::Rgb(220, 240, 80),
    gauge_cpu_bg: Color::Rgb(16, 32, 12),
    gauge_mem_bg: Color::Rgb(14, 28, 22),
    gauge_disk_bg: Color::Rgb(26, 20, 10),
};

pub const ARCTIC: Theme = Theme {
    name: "Arctic",
    meadow: Color::Rgb(100, 210, 230),
    clover: Color::Rgb(60, 150, 170),
    grass: Color::Rgb(160, 230, 240),
    milk: Color::Rgb(235, 245, 255),
    cream: Color::Rgb(200, 220, 235),
    daisy: Color::Rgb(255, 240, 200),
    blossom: Color::Rgb(240, 170, 200),
    sky: Color::Rgb(130, 200, 240),
    earth: Color::Rgb(180, 160, 140),
    spot: Color::Rgb(25, 30, 38),
    dim: Color::Rgb(120, 140, 160),
    bg: Color::Rgb(18, 24, 32),
    danger: Color::Rgb(255, 90, 90),
    warning: Color::Rgb(255, 210, 80),
    success: Color::Rgb(100, 210, 200),
    block_border: Color::Rgb(100, 210, 230),
    block_title: Color::Rgb(255, 240, 200),
    gauge_cpu_bg: Color::Rgb(22, 30, 40),
    gauge_mem_bg: Color::Rgb(20, 30, 42),
    gauge_disk_bg: Color::Rgb(28, 28, 36),
};

pub const RETRO: Theme = Theme {
    name: "Retro",
    meadow: Color::Rgb(0, 255, 0),
    clover: Color::Rgb(0, 180, 0),
    grass: Color::Rgb(80, 255, 80),
    milk: Color::Rgb(220, 255, 220),
    cream: Color::Rgb(180, 255, 180),
    daisy: Color::Rgb(255, 255, 0),
    blossom: Color::Rgb(255, 100, 255),
    sky: Color::Rgb(0, 200, 255),
    earth: Color::Rgb(180, 140, 60),
    spot: Color::Rgb(8, 12, 8),
    dim: Color::Rgb(80, 130, 80),
    bg: Color::Rgb(0, 4, 0),
    danger: Color::Rgb(255, 0, 0),
    warning: Color::Rgb(255, 255, 0),
    success: Color::Rgb(0, 255, 0),
    block_border: Color::Rgb(0, 255, 0),
    block_title: Color::Rgb(255, 255, 0),
    gauge_cpu_bg: Color::Rgb(8, 20, 8),
    gauge_mem_bg: Color::Rgb(8, 16, 24),
    gauge_disk_bg: Color::Rgb(20, 14, 8),
};

pub const ALL_THEMES: &[Theme] = &[PASTURE, MIDNIGHT, SUNSET, FOREST, ARCTIC, RETRO];
