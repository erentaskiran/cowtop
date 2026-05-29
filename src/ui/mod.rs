pub mod theme;
pub mod widgets;

mod cpu;
mod network;
mod overview;
mod processes;
mod sensors;
mod storage;

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, Tab};
use widgets::*;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let (banner_h, footer_h) = (9u16, 1u16);

    let chunks = Layout::vertical([
        Constraint::Length(banner_h),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(footer_h),
    ])
    .split(area);

    render_banner(frame, chunks[0], app);
    render_tabs(frame, chunks[1], app);

    // search bar (overlays the tab content area top line when active)
    let content_area = if app.search_query.is_some() {
        let c2 = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(chunks[2]);
        render_search_bar(frame, c2[0], app);
        c2[1]
    } else {
        chunks[2]
    };

    match app.tab {
        Tab::Overview => overview::render(frame, content_area, app),
        Tab::Cpu => cpu::render(frame, content_area, app),
        Tab::Processes => processes::render(frame, content_area, app),
        Tab::Network => network::render(frame, content_area, app),
        Tab::Storage => storage::render(frame, content_area, app),
        Tab::Sensors => sensors::render(frame, content_area, app),
    }

    // Help overlay
    if app.show_help {
        render_help(frame, frame.area(), app);
    }

    render_footer(frame, chunks[3], app);
}

fn render_banner(frame: &mut Frame, area: Rect, app: &App) {
    let mood = app.mood();
    let s = &app.snapshot;
    let t = &app.theme;

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(t.meadow))
        .title(Span::styled(
            " ✿ cowtop ✿ ",
            Style::default().fg(t.daisy).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .title_bottom(Span::styled(
            " ≋·≋·≋·✿·≋·≋·≋·✿·≋·≋·≋·✿·≋·≋·≋·✿·≋·≋·≋ ",
            Style::default().fg(t.clover),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cols = Layout::horizontal([Constraint::Length(38), Constraint::Min(0)]).split(inner);

    // Cow ASCII art
    let cow_lines: Vec<Line> = mood
        .art(app.cow_frame)
        .iter()
        .map(|l| {
            Line::from(Span::styled(
                *l,
                Style::default().fg(mood.color()).add_modifier(Modifier::BOLD),
            ))
        })
        .collect();
    frame.render_widget(Paragraph::new(cow_lines), cols[0]);

    // Right info column
    let cpu = s.cpu.total_percent;
    let mem = s.mem.used_percent();

    let hostname_str = if s.hostname.is_empty() { "—".to_string() } else { s.hostname.clone() };
    let kernel_str = truncate(&s.kernel, 45);

    let info = vec![
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                "the pasture system monitor",
                Style::default().fg(t.cream).add_modifier(Modifier::ITALIC),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("( {} )", mood.phrase()),
                Style::default().fg(mood.color()).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                format!(" {} ", mood.badge()),
                Style::default()
                    .fg(t.spot)
                    .bg(mood.color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  CPU ", Style::default().fg(t.dim)),
            Span::styled(format!("{:.0}%", cpu), Style::default().fg(t.gauge_color(cpu))),
            Span::styled("  MEM ", Style::default().fg(t.dim)),
            Span::styled(format!("{:.0}%", mem), Style::default().fg(t.gauge_color(mem))),
            Span::styled(
                format!("  load {:.2} {:.2} {:.2}", s.cpu.load1, s.cpu.load5, s.cpu.load15),
                Style::default().fg(t.cream),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ▼ rx ", Style::default().fg(t.dim)),
            Span::styled(fmt_bps(s.net.total_rx_bps), Style::default().fg(t.meadow)),
            Span::styled("  ▲ tx ", Style::default().fg(t.dim)),
            Span::styled(fmt_bps(s.net.total_tx_bps), Style::default().fg(Color::Cyan)),
            Span::styled("   disk r ", Style::default().fg(t.dim)),
            Span::styled(fmt_bps(s.disk_read_bps), Style::default().fg(Color::Yellow)),
            Span::styled(" w ", Style::default().fg(t.dim)),
            Span::styled(fmt_bps(s.disk_write_bps), Style::default().fg(Color::Magenta)),
        ]),
        Line::from(vec![
            Span::styled("  host ", Style::default().fg(t.dim)),
            Span::styled(&hostname_str as &str, Style::default().fg(t.daisy).add_modifier(Modifier::BOLD)),
            Span::styled("  kernel ", Style::default().fg(t.dim)),
            Span::styled(&kernel_str as &str, Style::default().fg(t.sky)),
        ]),
        Line::from(vec![
            Span::styled("  up ", Style::default().fg(t.dim)),
            Span::styled(fmt_uptime(s.cpu.uptime_seconds), Style::default().fg(t.cream)),
            Span::styled("   cores ", Style::default().fg(t.dim)),
            Span::styled(format!("{}", s.cpu.cores.len()), Style::default().fg(t.meadow)),
            Span::styled("   procs ", Style::default().fg(t.dim)),
            Span::styled(format!("{}", s.proc_total), Style::default().fg(t.meadow)),
            Span::styled("   ctx/s ", Style::default().fg(t.dim)),
            Span::styled(fmt_rate(app.ctx_rate), Style::default().fg(Color::Yellow)),
            Span::styled(" intr/s ", Style::default().fg(t.dim)),
            Span::styled(fmt_rate(app.intr_rate), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  theme ", Style::default().fg(t.dim)),
            Span::styled(t.name, Style::default().fg(t.daisy).add_modifier(Modifier::BOLD)),
            Span::styled("  t/T cycle ", Style::default().fg(t.dim)),
        ]),
    ];
    frame.render_widget(Paragraph::new(info), cols[1]);
}

fn render_tabs(frame: &mut Frame, area: Rect, app: &App) {
    let t = &app.theme;
    let tabs = Tabs::new(Tab::titles().to_vec())
        .select(Some(app.tab.index()))
        .style(Style::default().fg(t.dim))
        .highlight_style(
            Style::default()
                .fg(t.spot)
                .bg(t.meadow)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::styled(" ✿ ", Style::default().fg(t.clover)));
    frame.render_widget(tabs, area);
}

fn render_search_bar(frame: &mut Frame, area: Rect, app: &App) {
    let t = &app.theme;
    let q = app.search_query.as_deref().unwrap_or("");
    let bar = Paragraph::new(Line::from(vec![
        Span::styled(" / ", Style::default().fg(t.warning).add_modifier(Modifier::BOLD)),
        Span::styled(q, Style::default().fg(t.milk).add_modifier(Modifier::BOLD)),
        Span::styled(
            if q.is_empty() { " (type to filter, Esc to exit)" } else { "█" },
            Style::default().fg(t.dim),
        ),
    ]))
    .style(Style::default().bg(t.spot));
    frame.render_widget(bar, area);
}

fn render_help(frame: &mut Frame, area: Rect, app: &App) {
    let t = &app.theme;
    let help_lines = vec![
        Line::from(Span::styled(" Keys ", Style::default().fg(t.daisy).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled(" q / Esc  ", Style::default().fg(t.meadow).add_modifier(Modifier::BOLD)),
            Span::styled("quit", Style::default().fg(t.cream)),
        ]),
        Line::from(vec![
            Span::styled(" Tab / ←→ ", Style::default().fg(t.meadow).add_modifier(Modifier::BOLD)),
            Span::styled("switch tab", Style::default().fg(t.cream)),
        ]),
        Line::from(vec![
            Span::styled(" 1-6      ", Style::default().fg(t.meadow).add_modifier(Modifier::BOLD)),
            Span::styled("jump to tab", Style::default().fg(t.cream)),
        ]),
        Line::from(vec![
            Span::styled(" t / T    ", Style::default().fg(t.meadow).add_modifier(Modifier::BOLD)),
            Span::styled("next/prev theme", Style::default().fg(t.cream)),
        ]),
        Line::from(vec![
            Span::styled(" /        ", Style::default().fg(t.meadow).add_modifier(Modifier::BOLD)),
            Span::styled("search processes (Esc to cancel)", Style::default().fg(t.cream)),
        ]),
        Line::from(vec![
            Span::styled(" s        ", Style::default().fg(t.meadow).add_modifier(Modifier::BOLD)),
            Span::styled("cycle sort column (processes tab)", Style::default().fg(t.cream)),
        ]),
        Line::from(vec![
            Span::styled(" k        ", Style::default().fg(t.meadow).add_modifier(Modifier::BOLD)),
            Span::styled("kill process (select with enter)", Style::default().fg(t.cream)),
        ]),
        Line::from(vec![
            Span::styled(" p        ", Style::default().fg(t.meadow).add_modifier(Modifier::BOLD)),
            Span::styled("pause/resume", Style::default().fg(t.cream)),
        ]),
        Line::from(vec![
            Span::styled(" r        ", Style::default().fg(t.meadow).add_modifier(Modifier::BOLD)),
            Span::styled("force refresh", Style::default().fg(t.cream)),
        ]),
        Line::from(vec![
            Span::styled(" ↑↓ / jk  ", Style::default().fg(t.meadow).add_modifier(Modifier::BOLD)),
            Span::styled("scroll lists", Style::default().fg(t.cream)),
        ]),
        Line::from(vec![
            Span::styled(" ?        ", Style::default().fg(t.meadow).add_modifier(Modifier::BOLD)),
            Span::styled("toggle this help", Style::default().fg(t.cream)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            " Press ? or Esc to close ",
            Style::default().fg(t.dim),
        )),
    ];

    let h = help_lines.len() as u16 + 2;
    let w: u16 = 48;
    let popup_area = centered_rect(w, h, area);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(t.daisy))
        .title(Span::styled(" Help ", Style::default().fg(t.daisy).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(t.spot));

    frame.render_widget(Clear, popup_area);
    frame.render_widget(block.clone(), popup_area);
    frame.render_widget(Paragraph::new(help_lines), block.inner(popup_area));
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let t = &app.theme;
    let line = if let Some(err) = &app.error {
        Line::from(Span::styled(
            format!(" error: {} ", err),
            Style::default().fg(Color::White).bg(Color::Red),
        ))
    } else {
        let pause = if app.paused { " [PAUSED]" } else { "" };
        let search = if app.search_query.is_some() { " [SEARCH]" } else { "" };
        Line::from(vec![
            Span::styled(" q ", Style::default().fg(t.spot).bg(t.meadow)),
            Span::styled(" quit  ", Style::default().fg(t.dim)),
            Span::styled(" t ", Style::default().fg(t.spot).bg(t.meadow)),
            Span::styled(" theme  ", Style::default().fg(t.dim)),
            Span::styled(" ? ", Style::default().fg(t.spot).bg(t.meadow)),
            Span::styled(" help  ", Style::default().fg(t.dim)),
            Span::styled(" ↑↓ ", Style::default().fg(t.spot).bg(t.meadow)),
            Span::styled(" scroll  ", Style::default().fg(t.dim)),
            Span::styled(" p ", Style::default().fg(t.spot).bg(t.meadow)),
            Span::styled(" pause", Style::default().fg(t.dim)),
            Span::styled(pause, Style::default().fg(t.daisy).add_modifier(Modifier::BOLD)),
            Span::styled(search, Style::default().fg(t.warning).add_modifier(Modifier::BOLD)),
        ])
    };
    frame.render_widget(Paragraph::new(line), area);
}

fn centered_rect(w: u16, h: u16, area: Rect) -> Rect {
    Rect {
        x: area.x + (area.width.saturating_sub(w)) / 2,
        y: area.y + (area.height.saturating_sub(h)) / 2,
        width: w.min(area.width),
        height: h.min(area.height),
    }
}
