use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Gauge, Paragraph, Row, Sparkline, Table, Tabs},
    Frame,
};

use crate::app::{App, Series, Tab};

const PASTURE: Color = Color::Green;
const CREAM: Color = Color::Rgb(245, 240, 225);
const DIM: Color = Color::Rgb(120, 130, 120);

pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::vertical([
        Constraint::Length(8),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);

    render_banner(frame, chunks[0], app);
    render_tabs(frame, chunks[1], app);

    match app.tab {
        Tab::Overview => render_overview(frame, chunks[2], app),
        Tab::Cpu => render_cpu_tab(frame, chunks[2], app),
        Tab::Processes => render_proc_tab(frame, chunks[2], app),
        Tab::Network => render_net_tab(frame, chunks[2], app),
        Tab::Storage => render_storage_tab(frame, chunks[2], app),
    }

    render_footer(frame, chunks[3], app);
}

// ------------------------------------------------------------------ banner

fn render_banner(frame: &mut Frame, area: Rect, app: &App) {
    let mood = app.mood();
    let s = &app.snapshot;

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(PASTURE))
        .title(Span::styled(
            " cowtui ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cols = Layout::horizontal([Constraint::Length(36), Constraint::Min(0)]).split(inner);

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

    let cpu = s.cpu.total_percent;
    let mem = s.mem.used_percent();
    let info = vec![
        Line::from(vec![
            Span::styled(
                "the pasture system monitor",
                Style::default().fg(CREAM).add_modifier(Modifier::ITALIC),
            ),
        ]),
        Line::from(vec![Span::styled(
            format!("( {} )", mood.phrase()),
            Style::default().fg(mood.color()).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled(
                format!(" {} ", mood.badge()),
                Style::default()
                    .fg(Color::Black)
                    .bg(mood.color())
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("CPU ", Style::default().fg(DIM)),
            Span::styled(format!("{:.0}%", cpu), Style::default().fg(gauge_color(cpu))),
            Span::styled("  MEM ", Style::default().fg(DIM)),
            Span::styled(format!("{:.0}%", mem), Style::default().fg(gauge_color(mem))),
            Span::styled(
                format!("  load {:.2} {:.2} {:.2}", s.cpu.load1, s.cpu.load5, s.cpu.load15),
                Style::default().fg(CREAM),
            ),
        ]),
        Line::from(vec![
            Span::styled("net ", Style::default().fg(DIM)),
            Span::styled(format!("v{} ^{}", fmt_bps(s.net.total_rx_bps), fmt_bps(s.net.total_tx_bps)), Style::default().fg(Color::Cyan)),
            Span::styled("   disk ", Style::default().fg(DIM)),
            Span::styled(format!("r{} w{}", fmt_bps(s.disk_read_bps), fmt_bps(s.disk_write_bps)), Style::default().fg(Color::Magenta)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("up {}   ·   {} cores   ·   {} procs", fmt_uptime(s.cpu.uptime_seconds), s.cpu.cores.len(), s.proc_total),
                Style::default().fg(DIM),
            ),
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
                .fg(Color::Black)
                .bg(PASTURE)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::styled("·", Style::default().fg(DIM)));
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
            Span::styled(" q ", Style::default().fg(Color::Black).bg(PASTURE)),
            Span::styled(" quit  ", Style::default().fg(DIM)),
            Span::styled(" Tab/←→ ", Style::default().fg(Color::Black).bg(PASTURE)),
            Span::styled(" switch  ", Style::default().fg(DIM)),
            Span::styled(" 1-5 ", Style::default().fg(Color::Black).bg(PASTURE)),
            Span::styled(" jump  ", Style::default().fg(DIM)),
            Span::styled(" ↑↓ ", Style::default().fg(Color::Black).bg(PASTURE)),
            Span::styled(" scroll  ", Style::default().fg(DIM)),
            Span::styled(" p ", Style::default().fg(Color::Black).bg(PASTURE)),
            Span::styled(" pause", Style::default().fg(DIM)),
            Span::styled(pause, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ])
    };
    frame.render_widget(Paragraph::new(line), area);
}

// ------------------------------------------------------------------ overview

fn render_overview(frame: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::vertical([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .split(area);

    let r0 = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rows[0]);
    let r1 = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rows[1]);
    let r2 = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rows[2]);

    panel_cpu(frame, r0[0], app);
    panel_mem(frame, r0[1], app);
    panel_net(frame, r1[0], app);
    panel_disk(frame, r1[1], app);
    panel_packets(frame, r2[0], app);
    panel_top_procs(frame, r2[1], app);
}

fn pasture_block(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(PASTURE))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD),
        ))
}

fn panel_cpu(frame: &mut Frame, area: Rect, app: &App) {
    let block = pasture_block("CPU");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let parts = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(inner);
    let cpu = app.snapshot.cpu.total_percent;
    frame.render_widget(
        Gauge::default()
            .ratio((cpu / 100.0).clamp(0.0, 1.0))
            .label(format!("{:.1}%", cpu))
            .gauge_style(Style::default().fg(gauge_color(cpu)).bg(Color::Rgb(30, 35, 30))),
        parts[0],
    );
    frame.render_widget(sparkline(&app.cpu_hist, Color::Green, Some(100)), parts[1]);
}

fn panel_mem(frame: &mut Frame, area: Rect, app: &App) {
    let block = pasture_block("Memory pulse");
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
            .gauge_style(Style::default().fg(gauge_color(used)).bg(Color::Rgb(30, 35, 30))),
        parts[0],
    );
    let swap = mem.swap_percent();
    frame.render_widget(
        Gauge::default()
            .ratio((swap / 100.0).clamp(0.0, 1.0))
            .label(if mem.swap_total_kb == 0 {
                "swap none".to_string()
            } else {
                format!("swap {}/{}", fmt_kb(mem.swap_used_kb), fmt_kb(mem.swap_total_kb))
            })
            .gauge_style(Style::default().fg(Color::Blue).bg(Color::Rgb(30, 35, 30))),
        parts[1],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("avail ", Style::default().fg(DIM)),
            Span::styled(fmt_kb(mem.available_kb), Style::default().fg(CREAM)),
            Span::styled("  cache ", Style::default().fg(DIM)),
            Span::styled(fmt_kb(mem.cached_kb), Style::default().fg(CREAM)),
            Span::styled("  buf ", Style::default().fg(DIM)),
            Span::styled(fmt_kb(mem.buffers_kb), Style::default().fg(CREAM)),
        ])),
        parts[2],
    );
    frame.render_widget(sparkline(&app.mem_hist, Color::LightMagenta, Some(100)), parts[3]);
}

fn panel_net(frame: &mut Frame, area: Rect, app: &App) {
    let block = pasture_block("Network pulse");
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
            Span::styled("v rx ", Style::default().fg(DIM)),
            Span::styled(fmt_bps(net.total_rx_bps), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ])),
        parts[0],
    );
    frame.render_widget(sparkline(&app.rx_hist, Color::Green, None), parts[1]);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("^ tx ", Style::default().fg(DIM)),
            Span::styled(fmt_bps(net.total_tx_bps), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ])),
        parts[2],
    );
    frame.render_widget(sparkline(&app.tx_hist, Color::Cyan, None), parts[3]);
}

fn panel_disk(frame: &mut Frame, area: Rect, app: &App) {
    let block = pasture_block("Storage");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mounts = &app.snapshot.mounts;
    let n = mounts.len().min(inner.height.saturating_sub(1) as usize);
    let mut constraints: Vec<Constraint> = (0..n).map(|_| Constraint::Length(1)).collect();
    constraints.push(Constraint::Min(0));
    let parts = Layout::vertical(constraints).split(inner);

    for (i, m) in mounts.iter().take(n).enumerate() {
        frame.render_widget(
            Gauge::default()
                .ratio((m.used_percent / 100.0).clamp(0.0, 1.0))
                .label(format!("{} {:.0}% ({} free)", trim_mount(&m.mount), m.used_percent, fmt_kb(m.avail_kb)))
                .gauge_style(Style::default().fg(gauge_color(m.used_percent)).bg(Color::Rgb(30, 35, 30))),
            parts[i],
        );
    }
    if let Some(last) = parts.get(n) {
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("io  r ", Style::default().fg(DIM)),
                Span::styled(fmt_bps(app.snapshot.disk_read_bps), Style::default().fg(Color::Yellow)),
                Span::styled("  w ", Style::default().fg(DIM)),
                Span::styled(fmt_bps(app.snapshot.disk_write_bps), Style::default().fg(Color::Magenta)),
            ])),
            *last,
        );
    }
}

fn panel_packets(frame: &mut Frame, area: Rect, app: &App) {
    let block = pasture_block("Packet tracing");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let net = &app.snapshot.net;
    let lines = vec![
        Line::from(vec![
            Span::styled("TCP  ", Style::default().fg(DIM)),
            Span::styled(format!("estab {}", net.tcp_estab), Style::default().fg(Color::Green)),
            Span::raw("  "),
            Span::styled(format!("listen {}", net.tcp_listen), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("     "),
            Span::styled(format!("time_wait {}", net.tcp_time_wait), Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled(format!("other {}", net.tcp_other), Style::default().fg(DIM)),
        ]),
        Line::from(vec![
            Span::styled("UDP  ", Style::default().fg(DIM)),
            Span::styled(format!("{} sockets", net.udp_count), Style::default().fg(Color::Magenta)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{} sockets traced — see Packets tab", app.snapshot.conns.len()),
                Style::default().fg(DIM).add_modifier(Modifier::ITALIC),
            ),
        ]),
    ];
    frame.render_widget(Paragraph::new(lines), inner);
}

fn panel_top_procs(frame: &mut Frame, area: Rect, app: &App) {
    let block = pasture_block("Top herd (CPU)");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = app.snapshot.top_cpu.iter().take(inner.height as usize).map(|p| {
        Row::new(vec![
            format!("{:>6}", p.pid),
            format!("{:>5.1}%", p.cpu_percent),
            format!("{:>7}", fmt_kb(p.rss_kb)),
            truncate(&p.name, 18),
        ])
    });
    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Min(6),
        ],
    )
    .style(Style::default().fg(CREAM));
    frame.render_widget(table, inner);
}

// ------------------------------------------------------------------ CPU tab

fn render_cpu_tab(frame: &mut Frame, area: Rect, app: &App) {
    let block = pasture_block("CPU cores");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let cores = &app.snapshot.cpu.cores;
    if cores.is_empty() {
        return;
    }

    let head = Layout::vertical([Constraint::Length(2), Constraint::Min(1)]).split(inner);

    // Aggregate gauge + history at the top.
    let top = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(head[0]);
    let cpu = app.snapshot.cpu.total_percent;
    frame.render_widget(
        Gauge::default()
            .ratio((cpu / 100.0).clamp(0.0, 1.0))
            .label(format!("ALL {:.1}%", cpu))
            .gauge_style(Style::default().fg(gauge_color(cpu)).bg(Color::Rgb(30, 35, 30))),
        Rect { height: 1, ..top[0] },
    );
    frame.render_widget(sparkline(&app.cpu_hist, Color::Green, Some(100)), top[1]);

    // Per-core gauges in a few columns.
    let n = cores.len();
    let col_count = if n > 16 { 4 } else { 2 };
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
            frame.render_widget(
                Gauge::default()
                    .ratio((v / 100.0).clamp(0.0, 1.0))
                    .label(format!("c{:<2} {:>3.0}%", core_idx, v))
                    .gauge_style(Style::default().fg(gauge_color(v)).bg(Color::Rgb(28, 32, 28))),
                cells[i],
            );
        }
    }
}

// ------------------------------------------------------------------ Processes tab

fn render_proc_tab(frame: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);
    proc_table(frame, cols[0], "Top herd by CPU", &app.snapshot.top_cpu, app.table_scroll);
    proc_table(frame, cols[1], "Top herd by memory", &app.snapshot.top_mem, app.table_scroll);
}

fn proc_table(frame: &mut Frame, area: Rect, title: &str, procs: &[crate::sys::Proc], scroll: usize) {
    let block = pasture_block(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let header = Row::new(vec!["PID", "CPU%", "RSS", "COMMAND"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let visible = inner.height.saturating_sub(1) as usize;
    let rows = procs.iter().skip(scroll).take(visible).map(|p| {
        Row::new(vec![
            format!("{:>6}", p.pid),
            format!("{:>5.1}", p.cpu_percent),
            format!("{:>7}", fmt_kb(p.rss_kb)),
            truncate(&p.name, inner.width.saturating_sub(22) as usize),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Min(6),
        ],
    )
    .header(header)
    .style(Style::default().fg(CREAM));
    frame.render_widget(table, inner);
}

// ------------------------------------------------------------------ Network / packets tab

fn render_net_tab(frame: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::vertical([Constraint::Length(8), Constraint::Min(1)]).split(area);
    let top = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rows[0]);

    // Interfaces table.
    let block = pasture_block("Interfaces");
    let inner = block.inner(top[0]);
    frame.render_widget(block, top[0]);
    let header = Row::new(vec!["IFACE", "v RX/s", "^ TX/s", "TOTAL"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    let irows = app.snapshot.net.ifaces.iter().take(inner.height.saturating_sub(1) as usize).map(|i| {
        Row::new(vec![
            truncate(&i.name, 12),
            fmt_bps(i.rx_bps),
            fmt_bps(i.tx_bps),
            fmt_kb((i.rx_bytes + i.tx_bytes) / 1024),
        ])
    });
    frame.render_widget(
        Table::new(
            irows,
            [
                Constraint::Length(13),
                Constraint::Min(8),
                Constraint::Min(8),
                Constraint::Length(8),
            ],
        )
        .header(header)
        .style(Style::default().fg(CREAM)),
        inner,
    );

    // Net pulse sparklines.
    let block = pasture_block("Throughput pulse");
    let inner = block.inner(top[1]);
    frame.render_widget(block, top[1]);
    let sp = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .split(inner);
    frame.render_widget(Paragraph::new(Span::styled(format!("v rx {}", fmt_bps(app.snapshot.net.total_rx_bps)), Style::default().fg(Color::Green))), sp[0]);
    frame.render_widget(sparkline(&app.rx_hist, Color::Green, None), sp[1]);
    frame.render_widget(Paragraph::new(Span::styled(format!("^ tx {}", fmt_bps(app.snapshot.net.total_tx_bps)), Style::default().fg(Color::Cyan))), sp[2]);
    frame.render_widget(sparkline(&app.tx_hist, Color::Cyan, None), sp[3]);

    // Connection table (packet tracing).
    let net = &app.snapshot.net;
    let title = format!(
        "Sockets — tcp estab {} listen {} tw {} · udp {} · {} traced",
        net.tcp_estab, net.tcp_listen, net.tcp_time_wait, net.udp_count, app.snapshot.conns.len()
    );
    let block = pasture_block(&title);
    let inner = block.inner(rows[1]);
    frame.render_widget(block, rows[1]);
    let header = Row::new(vec!["PROTO", "LOCAL", "REMOTE", "STATE", "UID"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    let visible = inner.height.saturating_sub(1) as usize;
    let crows = app.snapshot.conns.iter().skip(app.table_scroll).take(visible).map(|c| {
        Row::new(vec![
            c.proto.clone(),
            truncate(&c.local, 24),
            truncate(&c.remote, 24),
            c.state.clone(),
            c.uid.to_string(),
        ])
        .style(Style::default().fg(conn_color(&c.state)))
    });
    frame.render_widget(
        Table::new(
            crows,
            [
                Constraint::Length(6),
                Constraint::Length(25),
                Constraint::Length(25),
                Constraint::Length(11),
                Constraint::Min(4),
            ],
        )
        .header(header)
        .style(Style::default().fg(CREAM)),
        inner,
    );
}

// ------------------------------------------------------------------ Storage tab

fn render_storage_tab(frame: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::vertical([Constraint::Min(1), Constraint::Length(4)]).split(area);

    let block = pasture_block("Filesystems");
    let inner = block.inner(rows[0]);
    frame.render_widget(block, rows[0]);

    let mounts = &app.snapshot.mounts;
    if !mounts.is_empty() {
        let n = mounts.len();
        let mut cons: Vec<Constraint> = (0..n).flat_map(|_| [Constraint::Length(1), Constraint::Length(1)]).collect();
        cons.push(Constraint::Min(0));
        let parts = Layout::vertical(cons).split(inner);
        for (i, m) in mounts.iter().enumerate() {
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(format!("{} ", m.mount), Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("[{}]  {}", m.fstype, m.source), Style::default().fg(DIM)),
                    Span::styled(format!("   {} used / {} total / {} free", fmt_kb(m.used_kb), fmt_kb(m.total_kb), fmt_kb(m.avail_kb)), Style::default().fg(CREAM)),
                ])),
                parts[i * 2],
            );
            frame.render_widget(
                Gauge::default()
                    .ratio((m.used_percent / 100.0).clamp(0.0, 1.0))
                    .label(format!("{:.1}%", m.used_percent))
                    .gauge_style(Style::default().fg(gauge_color(m.used_percent)).bg(Color::Rgb(30, 35, 30))),
                parts[i * 2 + 1],
            );
        }
    }

    let block = pasture_block("Disk IO pulse");
    let inner = block.inner(rows[1]);
    frame.render_widget(block, rows[1]);
    let sp = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(inner);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("read ", Style::default().fg(DIM)),
            Span::styled(fmt_bps(app.snapshot.disk_read_bps), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("   write ", Style::default().fg(DIM)),
            Span::styled(fmt_bps(app.snapshot.disk_write_bps), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        ])),
        sp[0],
    );
    frame.render_widget(sparkline(&app.disk_w_hist, Color::Magenta, None), sp[1]);
}

// ------------------------------------------------------------------ helpers

fn sparkline<'a>(series: &Series, color: Color, max: Option<u64>) -> Sparkline<'a> {
    let data = series.as_vec();
    let mut sp = Sparkline::default()
        .data(data)
        .style(Style::default().fg(color));
    if let Some(m) = max {
        sp = sp.max(m);
    }
    sp
}

fn gauge_color(pct: f64) -> Color {
    if pct >= 85.0 {
        Color::Red
    } else if pct >= 60.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

fn conn_color(state: &str) -> Color {
    match state {
        "ESTAB" => Color::Green,
        "LISTEN" => Color::Cyan,
        "TIME_WAIT" | "CLOSE_WAIT" | "CLOSING" => Color::Yellow,
        _ => CREAM,
    }
}

fn truncate(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

fn trim_mount(m: &str) -> String {
    let count = m.chars().count();
    if count <= 14 {
        m.to_string()
    } else {
        let tail: String = m.chars().skip(count - 13).collect();
        format!("…{}", tail)
    }
}

fn fmt_bps(bps: f64) -> String {
    if bps < 1024.0 {
        format!("{:.0} B/s", bps.max(0.0))
    } else if bps < 1024.0 * 1024.0 {
        format!("{:.1} KB/s", bps / 1024.0)
    } else if bps < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1} MB/s", bps / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB/s", bps / (1024.0 * 1024.0 * 1024.0))
    }
}

fn fmt_kb(kb: u64) -> String {
    let mb = kb as f64 / 1024.0;
    if mb < 1024.0 {
        format!("{:.0}M", mb)
    } else {
        format!("{:.1}G", mb / 1024.0)
    }
}

fn fmt_uptime(secs: f64) -> String {
    let s = secs as u64;
    let d = s / 86400;
    let h = (s % 86400) / 3600;
    let m = (s % 3600) / 60;
    if d > 0 {
        format!("{}d {}h {}m", d, h, m)
    } else {
        format!("{}h {}m", h, m)
    }
}
