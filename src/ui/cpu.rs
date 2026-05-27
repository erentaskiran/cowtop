use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::Gauge,
    Frame,
};

use crate::app::App;
use super::theme::*;
use super::widgets::*;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let block = cpu_block("CPU Cores");
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
            .gauge_style(Style::default().fg(gauge_color(cpu)).bg(Color::Rgb(28, 38, 25))),
        Rect { height: 1, ..top[0] },
    );
    frame.render_widget(sparkline(&app.cpu_hist, MEADOW, Some(100)), top[1]);

    let n = cores.len();
    let col_count = if n > 16 { 4 } else if n > 8 { 3 } else { 2 };
    let per_col = n.div_ceil(col_count);
    let col_constraints: Vec<Constraint> =
        (0..col_count).map(|_| Constraint::Ratio(1, col_count as u32)).collect();
    let columns = Layout::horizontal(col_constraints).split(head[1]);

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
            let color = if v >= 85.0 {
                Color::Red
            } else if v >= 60.0 {
                Color::Yellow
            } else if v >= 30.0 {
                MEADOW
            } else {
                CLOVER
            };
            frame.render_widget(
                Gauge::default()
                    .ratio((v / 100.0).clamp(0.0, 1.0))
                    .label(format!("c{:<2} {:>3.0}%", core_idx, v))
                    .gauge_style(Style::default().fg(color).bg(Color::Rgb(25, 35, 22))),
                cells[i],
            );
        }
    }
}
