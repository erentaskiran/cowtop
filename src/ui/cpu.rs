use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Gauge, Paragraph},
    Frame,
};

use crate::app::App;
use crate::cow::Mood;
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

    // Reserve the bottom of the tab for the grazing herd when there is room
    // (each core gets a cow). Falls back to all-gauges on short terminals.
    let herd_h = if inner.height >= 13 { 5 } else { 0 };
    let split = Layout::vertical([Constraint::Min(3), Constraint::Length(herd_h)]).split(inner);
    let inner = split[0];
    if herd_h > 0 {
        render_herd(frame, split[1], app);
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

/// The herd: one little cow per core, grazing or panicking with its load.
/// Cores that don't fit on one row are summarised as a "+N grazing" tail.
fn render_herd(frame: &mut Frame, area: Rect, app: &App) {
    let t = &app.theme;
    let block = t.pasture_block("The Herd");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.height < 3 {
        return;
    }

    let cores = &app.snapshot.cpu.cores;
    let cell_w: u16 = 6;
    let capacity = (inner.width / cell_w).max(1) as usize;
    let shown = cores.len().min(capacity);
    if shown == 0 {
        return;
    }

    let mut constraints: Vec<Constraint> = (0..shown).map(|_| Constraint::Length(cell_w)).collect();
    constraints.push(Constraint::Min(0));
    let cols = Layout::horizontal(constraints).split(inner);

    for (i, &load) in cores.iter().take(shown).enumerate() {
        let color = t.gauge_color(load);
        let cow = Mood::herd_cow(load);
        let para = Paragraph::new(vec![
            Line::from(Span::styled(cow[0], Style::default().fg(color))),
            Line::from(Span::styled(cow[1], Style::default().fg(color).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled(format!("c{:<2}", i), Style::default().fg(t.dim))),
        ]);
        frame.render_widget(para, cols[i]);
    }

    // Tail cell: note any cows still out in the back field.
    if let Some(tail) = cols.get(shown) {
        if cores.len() > shown {
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("+{} more", cores.len() - shown),
                        Style::default().fg(t.dim).add_modifier(Modifier::ITALIC),
                    )),
                ]),
                *tail,
            );
        }
    }
}
