use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use super::theme::*;
use super::widgets::*;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    // Üst satır: sistem bilgisi
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(area);

    let s = &app.snapshot;
    let info = Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{} procs total", s.proc_total), Style::default().fg(MEADOW)),
        Span::styled("  │  ", Style::default().fg(DIM)),
        Span::styled("ctx/s ", Style::default().fg(DIM)),
        Span::styled(fmt_rate(app.ctx_rate), Style::default().fg(Color::Yellow)),
        Span::styled("  intr/s ", Style::default().fg(DIM)),
        Span::styled(fmt_rate(app.intr_rate), Style::default().fg(Color::Yellow)),
        Span::styled("  │  ", Style::default().fg(DIM)),
        Span::styled("load ", Style::default().fg(DIM)),
        Span::styled(
            format!("{:.2} {:.2} {:.2}", s.cpu.load1, s.cpu.load5, s.cpu.load15),
            Style::default().fg(CREAM),
        ),
    ]);
    frame.render_widget(
        Paragraph::new(info).style(Style::default().bg(SPOT)),
        chunks[0],
    );

    let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(chunks[1]);
    proc_table(frame, cols[0], "Top by CPU", &app.snapshot.top_cpu, app.table_scroll, false);
    proc_table(frame, cols[1], "Top by MEM", &app.snapshot.top_mem, app.table_scroll, true);
}

fn proc_table(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    procs: &[crate::sys::Proc],
    scroll: usize,
    by_mem: bool,
) {
    let block = proc_block(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let header = if by_mem {
        Row::new(vec!["  PID", " MEM%", "   RSS", "CPU%", "COMMAND"])
            .style(Style::default().fg(DAISY).add_modifier(Modifier::BOLD))
    } else {
        Row::new(vec!["  PID", " CPU%", "   RSS", "COMMAND"])
            .style(Style::default().fg(DAISY).add_modifier(Modifier::BOLD))
    };

    let mem_total = if by_mem {
        // approximate
        procs.iter().map(|p| p.rss_kb).sum::<u64>().max(1)
    } else {
        1
    };

    let visible = inner.height.saturating_sub(1) as usize;
    let cmd_width = if by_mem {
        inner.width.saturating_sub(29) as usize
    } else {
        inner.width.saturating_sub(22) as usize
    };

    let rows: Vec<Row> = procs.iter().skip(scroll).take(visible).map(|p| {
        let row_color = if !by_mem && p.cpu_percent >= 50.0 {
            Color::Yellow
        } else if by_mem {
            let mem_pct = 100.0 * p.rss_kb as f64 / mem_total as f64;
            if mem_pct >= 10.0 { Color::Rgb(255, 180, 100) } else { CREAM }
        } else {
            CREAM
        };

        if by_mem {
            let mem_pct = 100.0 * p.rss_kb as f64 / mem_total as f64;
            Row::new(vec![
                format!("{:>6}", p.pid),
                format!("{:>5.1}", mem_pct),
                format!("{:>7}", fmt_kb(p.rss_kb)),
                format!("{:>4.1}", p.cpu_percent),
                truncate(&p.name, cmd_width),
            ])
            .style(Style::default().fg(row_color))
        } else {
            Row::new(vec![
                format!("{:>6}", p.pid),
                format!("{:>5.1}", p.cpu_percent),
                format!("{:>7}", fmt_kb(p.rss_kb)),
                truncate(&p.name, cmd_width),
            ])
            .style(Style::default().fg(row_color))
        }
    }).collect();

    let constraints = if by_mem {
        vec![
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(5),
            Constraint::Min(6),
        ]
    } else {
        vec![
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Min(6),
        ]
    };

    frame.render_widget(
        Table::new(rows, constraints).header(header),
        inner,
    );
}
