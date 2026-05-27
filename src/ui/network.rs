use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Paragraph, Row, Table},
    Frame,
};


use crate::app::App;
use super::theme::*;
use super::widgets::*;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::vertical([Constraint::Length(8), Constraint::Min(1)]).split(area);
    let top = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rows[0]);

    render_ifaces(frame, top[0], app);
    render_pulse(frame, top[1], app);
    render_conns(frame, rows[1], app);
}

fn render_ifaces(frame: &mut Frame, area: Rect, app: &App) {
    let block = net_block("Interfaces");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let header = Row::new(vec!["IFACE", "▼ RX/s", "▲ TX/s", "TOTAL"])
        .style(Style::default().fg(DAISY).add_modifier(Modifier::BOLD));

    let irows = app.snapshot.net.ifaces.iter()
        .take(inner.height.saturating_sub(1) as usize)
        .map(|i| {
            Row::new(vec![
                truncate(&i.name, 12),
                fmt_bps(i.rx_bps),
                fmt_bps(i.tx_bps),
                fmt_kb((i.rx_bytes + i.tx_bytes) / 1024),
            ])
            .style(Style::default().fg(CREAM))
        });

    frame.render_widget(
        Table::new(
            irows,
            [
                Constraint::Length(13),
                Constraint::Min(9),
                Constraint::Min(9),
                Constraint::Length(8),
            ],
        )
        .header(header),
        inner,
    );
}

fn render_pulse(frame: &mut Frame, area: Rect, app: &App) {
    let block = net_block("Throughput");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let sp = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .split(inner);

    frame.render_widget(
        Paragraph::new(Span::styled(
            format!("▼ rx  {}", fmt_bps(app.snapshot.net.total_rx_bps)),
            Style::default().fg(MEADOW).add_modifier(Modifier::BOLD),
        )),
        sp[0],
    );
    frame.render_widget(sparkline(&app.rx_hist, MEADOW, None), sp[1]);
    frame.render_widget(
        Paragraph::new(Span::styled(
            format!("▲ tx  {}", fmt_bps(app.snapshot.net.total_tx_bps)),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        sp[2],
    );
    frame.render_widget(sparkline(&app.tx_hist, Color::Cyan, None), sp[3]);
}

fn render_conns(frame: &mut Frame, area: Rect, app: &App) {
    let net = &app.snapshot.net;
    let title = format!(
        "Sockets  tcp {} estab · {} listen · {} tw  ·  udp {}  ·  {} traced",
        net.tcp_estab, net.tcp_listen, net.tcp_time_wait, net.udp_count,
        app.snapshot.conns.len()
    );
    let block = net_block(&title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let header = Row::new(vec!["PROTO", "LOCAL", "REMOTE", "STATE", "UID"])
        .style(Style::default().fg(DAISY).add_modifier(Modifier::BOLD));

    let visible = inner.height.saturating_sub(1) as usize;
    let crows = app.snapshot.conns.iter()
        .skip(app.table_scroll)
        .take(visible)
        .map(|c| {
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
        .header(header),
        inner,
    );
}

