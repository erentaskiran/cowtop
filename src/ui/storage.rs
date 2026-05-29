use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Gauge, Paragraph},
    Frame,
};

use crate::app::App;
use super::widgets::*;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::vertical([Constraint::Min(1), Constraint::Length(5)]).split(area);
    render_filesystems(frame, rows[0], app);
    render_io_pulse(frame, rows[1], app);
}

fn render_filesystems(frame: &mut Frame, area: Rect, app: &App) {
    let t = &app.theme;
    let block = t.disk_block("Filesystems");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mounts = &app.snapshot.mounts;
    if mounts.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled("  no mounts found", Style::default().fg(t.dim))),
            inner,
        );
        return;
    }

    let n = mounts.len();
    let mut cons: Vec<Constraint> = (0..n)
        .flat_map(|_| [Constraint::Length(1), Constraint::Length(1)])
        .collect();
    cons.push(Constraint::Min(0));
    let parts = Layout::vertical(cons).split(inner);

    for (i, m) in mounts.iter().enumerate() {
        let label_color = t.gauge_color(m.used_percent);

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(format!("{} ", m.mount), Style::default().fg(label_color).add_modifier(Modifier::BOLD)),
                Span::styled(format!("[{}] ", m.fstype), Style::default().fg(t.dim)),
                Span::styled(&m.source as &str, Style::default().fg(Color::Rgb(140, 155, 130))),
                Span::styled(
                    format!("   {} used  /  {} total  /  {} free", fmt_kb(m.used_kb), fmt_kb(m.total_kb), fmt_kb(m.avail_kb)),
                    Style::default().fg(t.cream),
                ),
            ])),
            parts[i * 2],
        );
        frame.render_widget(
            Gauge::default()
                .ratio((m.used_percent / 100.0).clamp(0.0, 1.0))
                .label(format!("{:.1}%", m.used_percent))
                .gauge_style(Style::default().fg(t.gauge_color(m.used_percent)).bg(t.gauge_disk_bg)),
            parts[i * 2 + 1],
        );
    }
}

fn render_io_pulse(frame: &mut Frame, area: Rect, app: &App) {
    let t = &app.theme;
    let block = t.disk_block("Disk IO");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let parts = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("read  ", Style::default().fg(t.dim)),
            Span::styled(fmt_bps(app.snapshot.disk_read_bps), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("     write  ", Style::default().fg(t.dim)),
            Span::styled(fmt_bps(app.snapshot.disk_write_bps), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ])),
        parts[0],
    );

    let half = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(parts[1]);
    frame.render_widget(sparkline(&app.disk_r_hist, Color::Yellow, None), half[0]);
    frame.render_widget(sparkline(&app.disk_w_hist, Color::Magenta, None), half[1]);
}
