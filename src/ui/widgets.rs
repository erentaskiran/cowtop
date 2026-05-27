use ratatui::{
    style::{Color, Style},
    widgets::Sparkline,
};

use crate::app::Series;

pub fn sparkline<'a>(series: &Series, color: Color, max: Option<u64>) -> Sparkline<'a> {
    let data = series.as_vec();
    let mut sp = Sparkline::default()
        .data(data)
        .style(Style::default().fg(color));
    if let Some(m) = max {
        sp = sp.max(m);
    }
    sp
}

pub fn fmt_bps(bps: f64) -> String {
    if bps < 1024.0 {
        format!("{:.0}B/s", bps.max(0.0))
    } else if bps < 1024.0 * 1024.0 {
        format!("{:.1}KB/s", bps / 1024.0)
    } else if bps < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1}MB/s", bps / (1024.0 * 1024.0))
    } else {
        format!("{:.2}GB/s", bps / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn fmt_kb(kb: u64) -> String {
    let mb = kb as f64 / 1024.0;
    if mb < 1024.0 {
        format!("{:.0}M", mb)
    } else {
        format!("{:.1}G", mb / 1024.0)
    }
}

pub fn fmt_uptime(secs: f64) -> String {
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

pub fn fmt_rate(n: u64) -> String {
    if n < 1_000 {
        format!("{}", n)
    } else if n < 1_000_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    }
}

pub fn truncate(s: &str, max: usize) -> String {
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

pub fn trim_mount(m: &str) -> String {
    let count = m.chars().count();
    if count <= 14 {
        m.to_string()
    } else {
        let tail: String = m.chars().skip(count - 13).collect();
        format!("…{}", tail)
    }
}
