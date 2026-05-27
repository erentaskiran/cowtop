use std::collections::VecDeque;

use crate::cow::Mood;
use crate::sys::{Monitor, Snapshot};

pub const HISTORY: usize = 240;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Cpu,
    Processes,
    Network,
    Storage,
}

impl Tab {
    pub const ALL: [Tab; 5] = [
        Tab::Overview,
        Tab::Cpu,
        Tab::Processes,
        Tab::Network,
        Tab::Storage,
    ];

    pub fn titles() -> [&'static str; 5] {
        [
            " Overview ",
            " CPU ",
            " Processes ",
            " Packets ",
            " Storage ",
        ]
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&t| t == self).unwrap_or(0)
    }
}

/// A fixed-length ring buffer of samples for sparklines.
pub struct Series {
    pub data: VecDeque<u64>,
}

impl Series {
    fn new() -> Self {
        Series {
            data: VecDeque::with_capacity(HISTORY),
        }
    }

    fn push(&mut self, value: u64) {
        if self.data.len() == HISTORY {
            self.data.pop_front();
        }
        self.data.push_back(value);
    }

    /// Most-recent-last slice, suitable for `Sparkline`.
    pub fn as_vec(&self) -> Vec<u64> {
        self.data.iter().copied().collect()
    }
}

pub struct App {
    monitor: Monitor,
    pub snapshot: Snapshot,
    pub tab: Tab,
    pub paused: bool,
    pub top_count: usize,
    pub table_scroll: usize,
    pub error: Option<String>,

    pub cpu_hist: Series,
    pub mem_hist: Series,
    pub rx_hist: Series,
    pub tx_hist: Series,
    pub disk_r_hist: Series,
    pub disk_w_hist: Series,

    pub ctx_rate: u64,
    pub intr_rate: u64,
}

impl App {
    pub fn new(monitor: Monitor, top_count: usize) -> Self {
        App {
            monitor,
            snapshot: Snapshot::default(),
            tab: Tab::Overview,
            paused: false,
            top_count,
            table_scroll: 0,
            error: None,
            cpu_hist: Series::new(),
            mem_hist: Series::new(),
            rx_hist: Series::new(),
            tx_hist: Series::new(),
            disk_r_hist: Series::new(),
            disk_w_hist: Series::new(),
            ctx_rate: 0,
            intr_rate: 0,
        }
    }

    pub fn refresh(&mut self) {
        if self.paused {
            return;
        }
        match self.monitor.sample(self.top_count) {
            Ok(s) => {
                self.ctx_rate = s.ctx_switches.saturating_sub(self.snapshot.ctx_switches);
                self.intr_rate = s.interrupts.saturating_sub(self.snapshot.interrupts);
                self.cpu_hist.push(s.cpu.total_percent.round() as u64);
                self.mem_hist.push(s.mem.used_percent().round() as u64);
                self.rx_hist.push((s.net.total_rx_bps / 1024.0).round() as u64);
                self.tx_hist.push((s.net.total_tx_bps / 1024.0).round() as u64);
                self.disk_r_hist.push((s.disk_read_bps / 1024.0).round() as u64);
                self.disk_w_hist.push((s.disk_write_bps / 1024.0).round() as u64);
                self.snapshot = s;
                self.error = None;
            }
            Err(e) => self.error = Some(e),
        }
    }

    pub fn mood(&self) -> Mood {
        Mood::from_load(
            self.snapshot.cpu.total_percent,
            self.snapshot.mem.used_percent(),
            self.snapshot.cpu.load1,
            self.snapshot.cpu.cores.len(),
        )
    }

    pub fn next_tab(&mut self) {
        let i = self.tab.index();
        self.tab = Tab::ALL[(i + 1) % Tab::ALL.len()];
        self.table_scroll = 0;
    }

    pub fn prev_tab(&mut self) {
        let i = self.tab.index();
        self.tab = Tab::ALL[(i + Tab::ALL.len() - 1) % Tab::ALL.len()];
        self.table_scroll = 0;
    }

    pub fn select_tab(&mut self, i: usize) {
        if i < Tab::ALL.len() {
            self.tab = Tab::ALL[i];
            self.table_scroll = 0;
        }
    }

    pub fn scroll_down(&mut self) {
        self.table_scroll = self.table_scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.table_scroll = self.table_scroll.saturating_sub(1);
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
}
