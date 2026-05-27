pub mod theme;
pub mod widgets;

mod overview;
mod cpu;
mod processes;
mod network;
mod storage;

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Tabs},
    Frame,
};

use crate::app::{App, Tab};
use theme::*;
use widgets::*;

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::vertical([
        Constraint::Length(9),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);

    render_banner(frame, chunks[0], app);
    render_tabs(frame, chunks[1], app);

    match app.tab {
        Tab::Overview => overview::render(frame, chunks[2], app),
        Tab::Cpu => cpu::render(frame, chunks[2], app),
        Tab::Processes => processes::render(frame, chunks[2], app),
        Tab::Network => network::render(frame, chunks[2], app),
        Tab::Storage => storage::render(frame, chunks[2], app),
    }

    render_footer(frame, chunks[3], app);
}

fn render_banner(frame: &mut Frame, area: Rect, app: &App) {
    let mood = app.mood();
    let s = &app.snapshot;

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MEADOW))
        .title(Span::styled(
            " ✿ cowtui ✿ ",
            Style::default().fg(DAISY).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .title_bottom(Span::styled(
            " ≋·≋·≋·✿·≋·≋·≋·✿·≋·≋·≋·✿·≋·≋·≋·✿·≋·≋·≋ ",
            Style::default().fg(CLOVER),
        ))
        ;

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cols = Layout::horizontal([Constraint::Length(38), Constraint::Min(0)]).split(inner);

    // İnek ASCII sanatı
    let cow_lines: Vec<Line> = mood
        .art()
        .iter()
        .map(|l| {
            Line::from(Span::styled(
                *l,
                Style::default().fg(mood.color()).add_modifier(Modifier::BOLD),
            ))
        })
        .collect();
    frame.render_widget(Paragraph::new(cow_lines), cols[0]);

    // Sağ bilgi sütunu
    let cpu = s.cpu.total_percent;
    let mem = s.mem.used_percent();

    let hostname_str = if s.hostname.is_empty() { "—".to_string() } else { s.hostname.clone() };
    let kernel_str = truncate(&s.kernel, 45);

    let info = vec![
        Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                "the pasture system monitor",
                Style::default().fg(CREAM).add_modifier(Modifier::ITALIC),
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
                    .fg(SPOT)
                    .bg(mood.color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  CPU ", Style::default().fg(DIM)),
            Span::styled(format!("{:.0}%", cpu), Style::default().fg(gauge_color(cpu))),
            Span::styled("  MEM ", Style::default().fg(DIM)),
            Span::styled(format!("{:.0}%", mem), Style::default().fg(gauge_color(mem))),
            Span::styled(
                format!("  load {:.2} {:.2} {:.2}", s.cpu.load1, s.cpu.load5, s.cpu.load15),
                Style::default().fg(CREAM),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ▼ rx ", Style::default().fg(DIM)),
            Span::styled(fmt_bps(s.net.total_rx_bps), Style::default().fg(MEADOW)),
            Span::styled("  ▲ tx ", Style::default().fg(DIM)),
            Span::styled(fmt_bps(s.net.total_tx_bps), Style::default().fg(Color::Cyan)),
            Span::styled("   disk r ", Style::default().fg(DIM)),
            Span::styled(fmt_bps(s.disk_read_bps), Style::default().fg(Color::Yellow)),
            Span::styled(" w ", Style::default().fg(DIM)),
            Span::styled(fmt_bps(s.disk_write_bps), Style::default().fg(Color::Magenta)),
        ]),
        Line::from(vec![
            Span::styled("  host ", Style::default().fg(DIM)),
            Span::styled(&hostname_str as &str, Style::default().fg(DAISY).add_modifier(Modifier::BOLD)),
            Span::styled("  kernel ", Style::default().fg(DIM)),
            Span::styled(&kernel_str as &str, Style::default().fg(SKY)),
        ]),
        Line::from(vec![
            Span::styled("  up ", Style::default().fg(DIM)),
            Span::styled(fmt_uptime(s.cpu.uptime_seconds), Style::default().fg(CREAM)),
            Span::styled("   cores ", Style::default().fg(DIM)),
            Span::styled(format!("{}", s.cpu.cores.len()), Style::default().fg(MEADOW)),
            Span::styled("   procs ", Style::default().fg(DIM)),
            Span::styled(format!("{}", s.proc_total), Style::default().fg(MEADOW)),
            Span::styled("   ctx/s ", Style::default().fg(DIM)),
            Span::styled(fmt_rate(app.ctx_rate), Style::default().fg(Color::Yellow)),
            Span::styled(" intr/s ", Style::default().fg(DIM)),
            Span::styled(fmt_rate(app.intr_rate), Style::default().fg(Color::Yellow)),
        ]),
    ];
    frame.render_widget(Paragraph::new(info), cols[1]);
}

fn render_tabs(frame: &mut Frame, area: Rect, app: &App) {
    let tabs = Tabs::new(Tab::titles().to_vec())
        .select(Some(app.tab.index()))
        .style(Style::default().fg(DIM))
        .highlight_style(
            Style::default()
                .fg(SPOT)
                .bg(MEADOW)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::styled(" ✿ ", Style::default().fg(CLOVER)));
    frame.render_widget(tabs, area);
}

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let line = if let Some(err) = &app.error {
        Line::from(Span::styled(
            format!(" error: {} ", err),
            Style::default().fg(Color::White).bg(Color::Red),
        ))
    } else {
        let pause = if app.paused { "  [PAUSED]" } else { "" };
        Line::from(vec![
            Span::styled(" q ", Style::default().fg(SPOT).bg(MEADOW)),
            Span::styled(" quit  ", Style::default().fg(DIM)),
            Span::styled(" Tab/←→ ", Style::default().fg(SPOT).bg(MEADOW)),
            Span::styled(" switch  ", Style::default().fg(DIM)),
            Span::styled(" 1-5 ", Style::default().fg(SPOT).bg(MEADOW)),
            Span::styled(" jump  ", Style::default().fg(DIM)),
            Span::styled(" ↑↓ ", Style::default().fg(SPOT).bg(MEADOW)),
            Span::styled(" scroll  ", Style::default().fg(DIM)),
            Span::styled(" p ", Style::default().fg(SPOT).bg(MEADOW)),
            Span::styled(" pause", Style::default().fg(DIM)),
            Span::styled(pause, Style::default().fg(DAISY).add_modifier(Modifier::BOLD)),
        ])
    };
    frame.render_widget(Paragraph::new(line), area);
}
