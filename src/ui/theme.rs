use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType},
};

// İnek temalı renk paleti
pub const MEADOW: Color = Color::Rgb(72, 158, 52);      // çimen yeşili – ana renk
pub const CLOVER: Color = Color::Rgb(45, 120, 38);       // koyu yonca
pub const GRASS: Color = Color::Rgb(120, 195, 85);       // açık çimen
pub const MILK: Color = Color::Rgb(248, 244, 234);       // süt beyazı
pub const CREAM: Color = Color::Rgb(230, 220, 195);      // krem
pub const DAISY: Color = Color::Rgb(255, 210, 50);       // papatya sarısı
pub const BLOSSOM: Color = Color::Rgb(255, 130, 165);    // çiçek pembe
pub const SKY: Color = Color::Rgb(90, 185, 225);         // gökyüzü mavisi
pub const EARTH: Color = Color::Rgb(140, 95, 50);        // toprak kahvesi
pub const SPOT: Color = Color::Rgb(35, 32, 28);          // inek leke siyahı
pub const DIM: Color = Color::Rgb(108, 122, 103);        // soluk yeşil

pub fn gauge_color(pct: f64) -> Color {
    if pct >= 85.0 {
        Color::Red
    } else if pct >= 60.0 {
        Color::Yellow
    } else {
        MEADOW
    }
}

pub fn conn_color(state: &str) -> Color {
    match state {
        "ESTAB" => MEADOW,
        "LISTEN" => SKY,
        "TIME_WAIT" | "CLOSE_WAIT" | "CLOSING" => Color::Yellow,
        _ => CREAM,
    }
}

// Standart inek-temalı panel
pub fn pasture_block(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MEADOW))
        .title(Span::styled(
            format!(" ✿ {} ✿ ", title),
            Style::default().fg(DAISY).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Left)
}

// CPU – yeşil kenarlık
pub fn cpu_block(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MEADOW))
        .title(Span::styled(
            format!(" ≋ {} ≋ ", title),
            Style::default().fg(GRASS).add_modifier(Modifier::BOLD),
        ))
        .title_bottom(Span::styled(
            " ·~·~·~·~·~·~·~·~· ",
            Style::default().fg(CLOVER),
        ))
}

// Memory – gökyüzü mavisi kenarlık
pub fn mem_block(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(SKY))
        .title(Span::styled(
            format!(" ✿ {} ✿ ", title),
            Style::default().fg(MILK).add_modifier(Modifier::BOLD),
        ))
        .title_bottom(Span::styled(
            " ·≋·≋·≋·≋·≋·≋·≋·≋· ",
            Style::default().fg(Color::Rgb(60, 150, 195)),
        ))
}

// Network – cyan kenarlık
pub fn net_block(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            format!(" ⊹ {} ⊹ ", title),
            Style::default().fg(SKY).add_modifier(Modifier::BOLD),
        ))
        .title_bottom(Span::styled(
            " ·⊹·⊹·⊹·⊹·⊹·⊹·⊹·⊹· ",
            Style::default().fg(Color::Rgb(50, 160, 185)),
        ))
}

// Storage – toprak/sarı kenarlık
pub fn disk_block(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(180, 130, 60)))
        .title(Span::styled(
            format!(" ⊕ {} ⊕ ", title),
            Style::default().fg(DAISY).add_modifier(Modifier::BOLD),
        ))
        .title_bottom(Span::styled(
            " ·⊕·⊕·⊕·⊕·⊕·⊕·⊕·⊕· ",
            Style::default().fg(EARTH),
        ))
}

// Processes – pembe kenarlık
pub fn proc_block(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BLOSSOM))
        .title(Span::styled(
            format!(" ※ {} ※ ", title),
            Style::default().fg(MILK).add_modifier(Modifier::BOLD),
        ))
        .title_bottom(Span::styled(
            " ·※·※·※·※·※·※·※·※· ",
            Style::default().fg(Color::Rgb(210, 90, 130)),
        ))
}
