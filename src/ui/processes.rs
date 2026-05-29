use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use super::widgets::*;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(area);

    let s = &app.snapshot;
    let t = &app.theme;
    let sort_label = match app.proc_sort {
        crate::app::ProcessSort::Cpu => "CPU",
        crate::app::ProcessSort::Mem => "MEM",
        crate::app::ProcessSort::Pid => "PID",
        crate::app::ProcessSort::Name => "NAME",
    };
    let dir_label = if app.proc_sort_desc { "desc" } else { "asc" };

    let info = Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{} procs total", s.proc_total), Style::default().fg(t.meadow)),
        Span::styled("  | ", Style::default().fg(t.dim)),
        Span::styled(format!("sort: {} {}", sort_label, dir_label), Style::default().fg(t.daisy)),
        Span::styled("  | ", Style::default().fg(t.dim)),
        Span::styled("load ", Style::default().fg(t.dim)),
        Span::styled(
            format!("{:.2} {:.2} {:.2}", s.cpu.load1, s.cpu.load5, s.cpu.load15),
            Style::default().fg(t.cream),
        ),
        Span::styled("  | ", Style::default().fg(t.dim)),
        Span::styled("/ search  ", Style::default().fg(t.dim)),
        Span::styled("s sort  ", Style::default().fg(t.dim)),
    ]);
    frame.render_widget(
        Paragraph::new(info).style(Style::default().bg(t.spot)),
        chunks[0],
    );

    let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(chunks[1]);
    proc_table(frame, cols[0], "Top by CPU", app, true);
    proc_table(frame, cols[1], "Top by MEM", app, false);
}

fn proc_table(frame: &mut Frame, area: Rect, title: &str, app: &App, by_cpu: bool) {
    let t = &app.theme;
    let block = t.proc_block(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let header = Row::new(vec![
        "  PID", "  PPID", "   CPU%", "   RSS", "S", "THR", "UID", "COMMAND",
    ])
    .style(Style::default().fg(t.daisy).add_modifier(Modifier::BOLD));

    let visible = inner.height.saturating_sub(1) as usize;
    let cmw = inner.width.saturating_sub(46) as usize;

    let procs = app.filtered_procs(visible, by_cpu);
    let rows: Vec<Row> = procs.iter().map(|p| {
        let row_color = if !by_cpu && p.cpu_percent >= 50.0 {
            t.warning
        } else {
            t.cream
        };

        Row::new(vec![
            format!("{:>6}", p.pid),
            format!("{:>6}", p.ppid),
            format!("{:>6.1}", p.cpu_percent),
            format!("{:>6}", fmt_kb(p.rss_kb)),
            p.state.to_string(),
            format!("{:>3}", p.threads),
            format!("{:>3}", p.uid),
            truncate(&p.name, cmw),
        ])
        .style(Style::default().fg(row_color))
    }).collect();

    frame.render_widget(
        Table::new(
            rows,
            [
                Constraint::Length(7),
                Constraint::Length(7),
                Constraint::Length(7),
                Constraint::Length(7),
                Constraint::Length(1),
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Min(6),
            ],
        )
        .header(header),
        inner,
    );
}
