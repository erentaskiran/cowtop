use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Gauge,
    Frame,
};

use crate::app::App;
use super::widgets::*;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let t = &app.theme;
    let block = t.cpu_block("CPU Cores");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cores = &app.snapshot.cpu.cores;
    if cores.is_empty() {
        return;
    }

    let head = Layout::vertical([Constraint::Length(2), Constraint::Min(1)]).split(inner);

    let top = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(head[0]);
    let cpu = app.snapshot.cpu.total_percent;
    frame.render_widget(
        Gauge::default()
            .ratio((cpu / 100.0).clamp(0.0, 1.0))
            .label(format!("ALL {:.1}%", cpu))
            .gauge_style(Style::default().fg(t.gauge_color(cpu)).bg(t.gauge_cpu_bg)),
        Rect { height: 1, ..top[0] },
    );
    frame.render_widget(sparkline(&app.cpu_hist, t.meadow, Some(100)), top[1]);

    // Show freq info if available
    let freqs = &app.snapshot.cpu_freqs;
    if !freqs.is_empty() {
        let freq_info = Layout::vertical([Constraint::Length(1)]).split(head[1]);
        let freq_spans: Vec<Span> = freqs.iter().take(16).flat_map(|f| {
            vec![
                Span::styled(
                    format!("c{}:", f.core_id),
                    Style::default().fg(t.dim),
                ),
                Span::styled(
                    format!("{:.0}MHz ", f.freq_mhz),
                    Style::default().fg(t.sky),
                ),
            ]
        }).collect();
        frame.render_widget(
            ratatui::widgets::Paragraph::new(Line::from(freq_spans)),
            freq_info[0],
        );

        let core_area = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(head[1]);
        render_cores(frame, core_area[1], app);
    } else {
        render_cores(frame, head[1], app);
    }
}

fn render_cores(frame: &mut Frame, area: Rect, app: &App) {
    let t = &app.theme;
    let cores = &app.snapshot.cpu.cores;
    let n = cores.len();
    let col_count = if n > 16 { 4 } else if n > 8 { 3 } else { 2 };
    let per_col = n.div_ceil(col_count);
    let col_constraints: Vec<Constraint> =
        (0..col_count).map(|_| Constraint::Ratio(1, col_count as u32)).collect();
    let columns = Layout::horizontal(col_constraints).split(area);

    for (c, col_area) in columns.iter().enumerate() {
        let start = c * per_col;
        let end = (start + per_col).min(n);
        if start >= end {
            continue;
        }
        let rows: Vec<Constraint> = (start..end).map(|_| Constraint::Length(1)).collect();
        let cells = Layout::vertical(rows).split(*col_area);
        for (i, core_idx) in (start..end).enumerate() {
            let v = cores[core_idx];
            let color = t.gauge_color(v);
            frame.render_widget(
                Gauge::default()
                    .ratio((v / 100.0).clamp(0.0, 1.0))
                    .label(format!("c{:<2} {:>3.0}%", core_idx, v))
                    .gauge_style(Style::default().fg(color).bg(t.gauge_cpu_bg)),
                cells[i],
            );
        }
    }
}
