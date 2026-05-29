use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Gauge, Paragraph},
    Frame,
};

use crate::app::App;
use super::theme::Theme;
use super::widgets::*;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Min(1),
        Constraint::Min(1),
    ])
    .split(area);

    render_freqs(frame, chunks[0], app);
    render_thermals(frame, chunks[1], app);
}

fn render_freqs(frame: &mut Frame, area: Rect, app: &App) {
    let t = &app.theme;
    let block = t.sensor_block("CPU Frequencies");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let freqs = &app.snapshot.cpu_freqs;
    if freqs.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled("  no frequency data (check cpufreq)", Style::default().fg(t.dim))),
            inner,
        );
        return;
    }

    let cores = &app.snapshot.cpu.cores;
    let n = freqs.len();
    let col_count = if n > 16 { 4 } else if n > 8 { 3 } else { 2 };
    let per_col = n.div_ceil(col_count);
    let col_constraints: Vec<Constraint> =
        (0..col_count).map(|_| Constraint::Ratio(1, col_count as u32)).collect();
    let columns = Layout::horizontal(col_constraints).split(inner);

    for (c, col_area) in columns.iter().enumerate() {
        let start = c * per_col;
        let end = (start + per_col).min(n);
        if start >= end {
            continue;
        }
        let rows: Vec<Constraint> = (start..end).flat_map(|_| {
            [Constraint::Length(1), Constraint::Length(1)]
        }).collect();
        let cells = Layout::vertical(rows).split(*col_area);
        for (i, idx) in (start..end).enumerate() {
            let f = &freqs[idx];
            let core_pct = cores.get(idx).copied().unwrap_or(0.0);
            let color = t.gauge_color(core_pct);

            let label = format!("c{}  {:.0} MHz  {:.0}%", f.core_id, f.freq_mhz, core_pct);
            frame.render_widget(
                Gauge::default()
                    .ratio((core_pct / 100.0).clamp(0.0, 1.0))
                    .label(label)
                    .gauge_style(Style::default().fg(color).bg(t.gauge_cpu_bg)),
                cells[i * 2],
            );

            let bar_label = if f.freq_mhz > 0.0 {
                format!("{:.0} / 6000 MHz", f.freq_mhz)
            } else {
                "—".to_string()
            };
            let max_freq = 6000.0;
            frame.render_widget(
                Gauge::default()
                    .ratio((f.freq_mhz / max_freq).clamp(0.0, 1.0))
                    .label(bar_label)
                    .gauge_style(Style::default().fg(t.sky).bg(t.gauge_mem_bg)),
                cells[i * 2 + 1],
            );
        }
    }
}

fn render_thermals(frame: &mut Frame, area: Rect, app: &App) {
    let t = &app.theme;
    let block = t.sensor_block("Thermal Zones");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let thermals = &app.snapshot.thermals;
    if thermals.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled("  no thermal zones found", Style::default().fg(t.dim))),
            inner,
        );
        return;
    }

    let n = thermals.len().min(inner.height.saturating_sub(1) as usize);
    if n == 0 {
        return;
    }
    let row_constraints: Vec<Constraint> = (0..n).map(|_| Constraint::Length(1)).collect();
    let rows = Layout::vertical(row_constraints).split(inner);

    for (i, th) in thermals.iter().take(n).enumerate() {
        let temp_color = t.temp_color(th.temp_c);

        let mut spans = vec![
            Span::styled(
                format!(" {} ", truncate(&th.name, 20)),
                Style::default().fg(t.daisy).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:.1}C  ", th.temp_c),
                Style::default().fg(temp_color).add_modifier(Modifier::BOLD),
            ),
        ];
        spans.extend(gauge_spans(t, th.temp_c, 100.0, 40));
        frame.render_widget(Paragraph::new(Line::from(spans)), rows[i]);
    }
}

fn gauge_spans(theme: &Theme, value: f64, max: f64, width: usize) -> Vec<Span<'static>> {
    let ratio = (value / max).clamp(0.0, 1.0);
    let filled = (ratio * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    let color = theme.gauge_color(ratio * 100.0);

    let mut bar = String::with_capacity(filled + empty);
    for _ in 0..filled { bar.push('█'); }
    for _ in 0..empty { bar.push('░'); }
    vec![Span::styled(bar, Style::default().fg(color))]
}
