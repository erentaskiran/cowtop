use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Gauge, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use super::theme::*;
use super::widgets::*;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(1),   // sistem bilgi çubuğu
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .split(area);

    render_sys_bar(frame, chunks[0], app);

    let r1 = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(chunks[1]);
    let r2 = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(chunks[2]);
    let r3 = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(chunks[3]);

    panel_cpu(frame, r1[0], app);
    panel_mem(frame, r1[1], app);
    panel_net(frame, r2[0], app);
    panel_disk(frame, r2[1], app);
    panel_top_cpu(frame, r3[0], app);
    panel_top_mem(frame, r3[1], app);
}

fn render_sys_bar(frame: &mut Frame, area: Rect, app: &App) {
    let s = &app.snapshot;
    let line = Line::from(vec![
        Span::raw("  "),
        Span::styled(
            if s.hostname.is_empty() { "localhost".to_string() } else { s.hostname.clone() },
            Style::default().fg(DAISY).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  │  ", Style::default().fg(Color::Rgb(60, 90, 55))),
        Span::styled(
            truncate(&s.kernel, 40),
            Style::default().fg(SKY),
        ),
        Span::styled("  │  ", Style::default().fg(Color::Rgb(60, 90, 55))),
        Span::styled("up ", Style::default().fg(DIM)),
        Span::styled(fmt_uptime(s.cpu.uptime_seconds), Style::default().fg(CREAM)),
        Span::styled("  │  ", Style::default().fg(Color::Rgb(60, 90, 55))),
        Span::styled(format!("{} procs", s.proc_total), Style::default().fg(MEADOW)),
        Span::styled("  │  ", Style::default().fg(Color::Rgb(60, 90, 55))),
        Span::styled("ctx/s ", Style::default().fg(DIM)),
        Span::styled(fmt_rate(app.ctx_rate), Style::default().fg(Color::Yellow)),
        Span::styled("  intr/s ", Style::default().fg(DIM)),
        Span::styled(fmt_rate(app.intr_rate), Style::default().fg(Color::Yellow)),
    ]);
    frame.render_widget(
        Paragraph::new(line).style(Style::default().bg(SPOT).fg(MILK)),
        area,
    );
}

fn panel_cpu(frame: &mut Frame, area: Rect, app: &App) {
    let block = cpu_block("CPU");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let parts = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(inner);
    let cpu = app.snapshot.cpu.total_percent;
    frame.render_widget(
        Gauge::default()
            .ratio((cpu / 100.0).clamp(0.0, 1.0))
            .label(format!(
                "{:.1}%  load {:.2} {:.2} {:.2}",
                cpu,
                app.snapshot.cpu.load1,
                app.snapshot.cpu.load5,
                app.snapshot.cpu.load15
            ))
            .gauge_style(Style::default().fg(gauge_color(cpu)).bg(Color::Rgb(28, 38, 25))),
        parts[0],
    );
    frame.render_widget(sparkline(&app.cpu_hist, MEADOW, Some(100)), parts[1]);
}

fn panel_mem(frame: &mut Frame, area: Rect, app: &App) {
    let block = mem_block("Memory");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mem = &app.snapshot.mem;
    let parts = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .split(inner);

    let used = mem.used_percent();
    frame.render_widget(
        Gauge::default()
            .ratio((used / 100.0).clamp(0.0, 1.0))
            .label(format!("RAM {}/{}", fmt_kb(mem.used_kb), fmt_kb(mem.total_kb)))
            .gauge_style(Style::default().fg(gauge_color(used)).bg(Color::Rgb(25, 35, 45))),
        parts[0],
    );
    let swap = mem.swap_percent();
    frame.render_widget(
        Gauge::default()
            .ratio((swap / 100.0).clamp(0.0, 1.0))
            .label(if mem.swap_total_kb == 0 {
                "swap —".to_string()
            } else {
                format!("swap {}/{}", fmt_kb(mem.swap_used_kb), fmt_kb(mem.swap_total_kb))
            })
            .gauge_style(Style::default().fg(Color::Rgb(80, 160, 210)).bg(Color::Rgb(25, 35, 45))),
        parts[1],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("avail ", Style::default().fg(DIM)),
            Span::styled(fmt_kb(mem.available_kb), Style::default().fg(MILK)),
            Span::styled("  cache ", Style::default().fg(DIM)),
            Span::styled(fmt_kb(mem.cached_kb), Style::default().fg(MILK)),
            Span::styled("  buf ", Style::default().fg(DIM)),
            Span::styled(fmt_kb(mem.buffers_kb), Style::default().fg(MILK)),
        ])),
        parts[2],
    );
    frame.render_widget(sparkline(&app.mem_hist, SKY, Some(100)), parts[3]);
}

fn panel_net(frame: &mut Frame, area: Rect, app: &App) {
    let block = net_block("Network");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let net = &app.snapshot.net;
    let parts = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("▼ rx ", Style::default().fg(DIM)),
            Span::styled(
                fmt_bps(net.total_rx_bps),
                Style::default().fg(MEADOW).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "  tcp:{} listen:{} tw:{} udp:{}",
                    net.tcp_estab, net.tcp_listen, net.tcp_time_wait, net.udp_count
                ),
                Style::default().fg(DIM),
            ),
        ])),
        parts[0],
    );
    frame.render_widget(sparkline(&app.rx_hist, MEADOW, None), parts[1]);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("▲ tx ", Style::default().fg(DIM)),
            Span::styled(
                fmt_bps(net.total_tx_bps),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ])),
        parts[2],
    );
    frame.render_widget(sparkline(&app.tx_hist, Color::Cyan, None), parts[3]);
}

fn panel_disk(frame: &mut Frame, area: Rect, app: &App) {
    let block = disk_block("Storage");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mounts = &app.snapshot.mounts;
    let n = mounts.len().min(inner.height.saturating_sub(2) as usize);
    let mut constraints: Vec<Constraint> = (0..n).map(|_| Constraint::Length(1)).collect();
    constraints.push(Constraint::Length(1));
    constraints.push(Constraint::Min(0));
    let parts = Layout::vertical(constraints).split(inner);

    for (i, m) in mounts.iter().take(n).enumerate() {
        frame.render_widget(
            Gauge::default()
                .ratio((m.used_percent / 100.0).clamp(0.0, 1.0))
                .label(format!(
                    "{} {:.0}% ({} free)",
                    trim_mount(&m.mount),
                    m.used_percent,
                    fmt_kb(m.avail_kb)
                ))
                .gauge_style(Style::default().fg(gauge_color(m.used_percent)).bg(Color::Rgb(38, 28, 18))),
            parts[i],
        );
    }
    if let (Some(io_area), Some(_)) = (parts.get(n), parts.get(n + 1)) {
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("io  r ", Style::default().fg(DIM)),
                Span::styled(
                    fmt_bps(app.snapshot.disk_read_bps),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled("  w ", Style::default().fg(DIM)),
                Span::styled(
                    fmt_bps(app.snapshot.disk_write_bps),
                    Style::default().fg(Color::Magenta),
                ),
            ])),
            *io_area,
        );
    }
}

fn panel_top_cpu(frame: &mut Frame, area: Rect, app: &App) {
    let block = proc_block("Top by CPU");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let header = Row::new(vec!["  PID", "CPU%", "   RSS", "COMMAND"])
        .style(Style::default().fg(DAISY).add_modifier(Modifier::BOLD));

    let visible = inner.height.saturating_sub(1) as usize;
    let rows = app.snapshot.top_cpu.iter().take(visible).map(|p| {
        Row::new(vec![
            format!("{:>6}", p.pid),
            format!("{:>5.1}", p.cpu_percent),
            format!("{:>7}", fmt_kb(p.rss_kb)),
            truncate(&p.name, inner.width.saturating_sub(22) as usize),
        ])
        .style(Style::default().fg(if p.cpu_percent >= 50.0 {
            Color::Yellow
        } else {
            CREAM
        }))
    });

    frame.render_widget(
        Table::new(
            rows,
            [
                Constraint::Length(7),
                Constraint::Length(6),
                Constraint::Length(8),
                Constraint::Min(6),
            ],
        )
        .header(header),
        inner,
    );
}

fn panel_top_mem(frame: &mut Frame, area: Rect, app: &App) {
    let block = proc_block("Top by MEM");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let header = Row::new(vec!["  PID", " MEM%", "   RSS", "COMMAND"])
        .style(Style::default().fg(DAISY).add_modifier(Modifier::BOLD));

    let visible = inner.height.saturating_sub(1) as usize;
    let rows = app.snapshot.top_mem.iter().take(visible).map(|p| {
        let mem_pct = if app.snapshot.mem.total_kb > 0 {
            100.0 * p.rss_kb as f64 / app.snapshot.mem.total_kb as f64
        } else {
            0.0
        };
        Row::new(vec![
            format!("{:>6}", p.pid),
            format!("{:>5.1}", mem_pct),
            format!("{:>7}", fmt_kb(p.rss_kb)),
            truncate(&p.name, inner.width.saturating_sub(22) as usize),
        ])
        .style(Style::default().fg(if mem_pct >= 10.0 {
            Color::Rgb(255, 180, 100)
        } else {
            CREAM
        }))
    });

    frame.render_widget(
        Table::new(
            rows,
            [
                Constraint::Length(7),
                Constraint::Length(6),
                Constraint::Length(8),
                Constraint::Min(6),
            ],
        )
        .header(header),
        inner,
    );
}
