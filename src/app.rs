use std::collections::VecDeque;
use std::path::PathBuf;

use crate::cow::Mood;
use crate::sys::{Monitor, Snapshot};
use crate::ui::theme::{Theme, ALL_THEMES, PASTURE};

pub const HISTORY: usize = 240;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Overview,
    Cpu,
    Processes,
    Network,
    Storage,
    Sensors,
}

impl Tab {
    pub const ALL: [Tab; 6] = [
        Tab::Overview,
        Tab::Cpu,
        Tab::Processes,
        Tab::Network,
        Tab::Storage,
        Tab::Sensors,
    ];

    pub fn titles() -> [&'static str; 6] {
        [
            " Overview ",
            " CPU ",
            " Processes ",
            " Packets ",
            " Storage ",
            " Sensors ",
        ]
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&t| t == self).unwrap_or(0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProcessSort {
    Cpu,
    Mem,
    Pid,
    Name,
}

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

    // Theme
    pub theme: Theme,

    // Process search / sort
    pub search_query: Option<String>,
    pub lowered_query: String,
    pub proc_sort: ProcessSort,
    pub proc_sort_desc: bool,
    pub selected_pid: Option<i32>,

    // Help overlay
    pub show_help: bool,

    // Cow animation frame
    pub cow_frame: u64,
}

impl App {
    pub fn new(monitor: Monitor, top_count: usize) -> Self {
        let theme = Self::load_config().unwrap_or(PASTURE.clone());

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
            theme,
            search_query: None,
            lowered_query: String::new(),
            proc_sort: ProcessSort::Cpu,
            proc_sort_desc: true,
            selected_pid: None,
            show_help: false,
            cow_frame: 0,
        }
    }

    pub fn refresh(&mut self) {
        if self.paused {
            return;
        }
        self.cow_frame = self.cow_frame.wrapping_add(1);
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

    fn theme_idx(&self) -> usize {
        ALL_THEMES.iter().position(|t| t.name == self.theme.name).unwrap_or(0)
    }

    pub fn next_theme(&mut self) {
        let i = (self.theme_idx() + 1) % ALL_THEMES.len();
        self.theme = ALL_THEMES[i].clone();
        self.save_config();
    }

    pub fn prev_theme(&mut self) {
        let i = (self.theme_idx() + ALL_THEMES.len() - 1) % ALL_THEMES.len();
        self.theme = ALL_THEMES[i].clone();
        self.save_config();
    }

    fn config_path() -> PathBuf {
        dirs_next().unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("cowtop")
            .join("config.json")
    }

    fn load_config() -> Option<Theme> {
        let path = Self::config_path();
        let data = std::fs::read_to_string(&path).ok()?;
        let name = data.trim();
        ALL_THEMES.iter().find(|t| t.name == name).cloned()
    }

    fn save_config(&self) {
        if let Some(parent) = Self::config_path().parent() {
            let _ = std::fs::create_dir_all(parent);
            let _ = std::fs::write(Self::config_path(), self.theme.name.as_bytes());
        }
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

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        self.search_query = None;
    }

    pub fn enter_search(&mut self) {
        self.search_query = Some(String::new());
        self.lowered_query.clear();
    }

    pub fn exit_search(&mut self) {
        self.search_query = None;
        self.lowered_query.clear();
    }

    pub fn search_push(&mut self, c: char) {
        if let Some(ref mut q) = self.search_query {
            q.push(c);
            self.lowered_query = q.to_lowercase();
        }
    }

    pub fn search_pop(&mut self) {
        if let Some(ref mut q) = self.search_query {
            q.pop();
            self.lowered_query = q.to_lowercase();
        }
    }

    pub fn cycle_sort(&mut self) {
        self.proc_sort = match self.proc_sort {
            ProcessSort::Cpu => ProcessSort::Mem,
            ProcessSort::Mem => ProcessSort::Pid,
            ProcessSort::Pid => ProcessSort::Name,
            ProcessSort::Name => ProcessSort::Cpu,
        };
        self.table_scroll = 0;
    }

    /// Return processes filtered by search query and sorted by current column.
    pub fn filtered_procs(&self, top_n: usize, by_cpu: bool) -> Vec<crate::sys::Proc> {
        let source = if by_cpu { &self.snapshot.top_cpu } else { &self.snapshot.top_mem };

        // Filter: apply before cloning to avoid allocating filtered-out strings
        let q = &self.lowered_query;
        let mut procs: Vec<crate::sys::Proc> = if q.is_empty() {
            source.clone()
        } else {
            source.iter()
                .filter(|p| p.name.to_lowercase().contains(q) || p.pid.to_string().contains(q))
                .cloned()
                .collect()
        };

        // sort
        match self.proc_sort {
            ProcessSort::Cpu => {
                procs.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));
            }
            ProcessSort::Mem => {
                procs.sort_by(|a, b| b.rss_kb.cmp(&a.rss_kb));
            }
            ProcessSort::Pid => {
                procs.sort_by(|a, b| a.pid.cmp(&b.pid));
            }
            ProcessSort::Name => {
                procs.sort_by(|a, b| a.name.cmp(&b.name));
            }
        }
        if !self.proc_sort_desc {
            procs.reverse();
        }

        procs.truncate(top_n);
        procs
    }
}

fn dirs_next() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}
