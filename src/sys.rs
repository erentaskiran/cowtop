//! Safe wrapper over the C monitor. Owns the `CowMonitor` handle and converts
//! the flat FFI sample into owned, UI-friendly Rust structs.

use std::ffi::CString;
use std::mem::MaybeUninit;
use std::os::raw::c_char;

use crate::ffi;

fn c_str(buf: &[c_char]) -> String {
    let bytes: Vec<u8> = buf
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

#[derive(Clone, Default)]
pub struct Cpu {
    pub total_percent: f64,
    pub cores: Vec<f64>,
    pub load1: f64,
    pub load5: f64,
    pub load15: f64,
    pub uptime_seconds: f64,
}

#[derive(Clone, Default)]
pub struct Mem {
    pub total_kb: u64,
    pub used_kb: u64,
    pub available_kb: u64,
    pub buffers_kb: u64,
    pub cached_kb: u64,
    pub swap_total_kb: u64,
    pub swap_used_kb: u64,
}

impl Mem {
    pub fn used_percent(&self) -> f64 {
        if self.total_kb == 0 {
            0.0
        } else {
            100.0 * self.used_kb as f64 / self.total_kb as f64
        }
    }

    pub fn swap_percent(&self) -> f64 {
        if self.swap_total_kb == 0 {
            0.0
        } else {
            100.0 * self.swap_used_kb as f64 / self.swap_total_kb as f64
        }
    }
}

#[derive(Clone)]
pub struct Iface {
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_bps: f64,
    pub tx_bps: f64,
}

#[derive(Clone, Default)]
pub struct Net {
    pub ifaces: Vec<Iface>,
    pub total_rx_bps: f64,
    pub total_tx_bps: f64,
    pub tcp_estab: i32,
    pub tcp_listen: i32,
    pub tcp_time_wait: i32,
    pub tcp_other: i32,
    pub udp_count: i32,
}

#[derive(Clone)]
pub struct Conn {
    pub proto: String,
    pub local: String,
    pub remote: String,
    pub state: String,
    pub uid: i32,
}

#[derive(Clone)]
pub struct Mount {
    pub source: String,
    pub mount: String,
    pub fstype: String,
    pub total_kb: u64,
    pub used_kb: u64,
    pub avail_kb: u64,
    pub used_percent: f64,
}

#[derive(Clone)]
pub struct Proc {
    pub pid: i32,
    pub ppid: i32,
    pub name: String,
    pub state: char,
    pub threads: i32,
    pub uid: i32,
    pub cpu_percent: f64,
    pub rss_kb: u64,
}

#[derive(Clone)]
pub struct CpuFreq {
    pub core_id: i32,
    pub freq_mhz: f64,
}

#[derive(Clone)]
pub struct Thermal {
    pub name: String,
    pub temp_c: f64,
}

#[derive(Clone, Default)]
pub struct Snapshot {
    pub cpu: Cpu,
    pub mem: Mem,
    pub net: Net,
    pub disk_read_bps: f64,
    pub disk_write_bps: f64,
    pub mounts: Vec<Mount>,
    pub conns: Vec<Conn>,
    pub top_cpu: Vec<Proc>,
    pub top_mem: Vec<Proc>,
    pub proc_total: u32,
    pub hostname: String,
    pub kernel: String,
    pub ctx_switches: u64,
    pub interrupts: u64,
    pub cpu_freqs: Vec<CpuFreq>,
    pub thermals: Vec<Thermal>,
}

pub struct Monitor {
    handle: *mut ffi::CowMonitor,
}

#[derive(Clone, Copy)]
pub enum Signal {
    Term,
    Kill,
    Int,
    Hup,
}

impl Monitor {
    pub fn kill_process(pid: i32, sig: Signal) -> Result<(), String> {
        let sig_no = match sig {
            Signal::Term => libc::SIGTERM,
            Signal::Kill => libc::SIGKILL,
            Signal::Int => libc::SIGINT,
            Signal::Hup => libc::SIGHUP,
        };
        let rc = unsafe { libc::kill(pid, sig_no) };
        if rc != 0 {
            Err(std::io::Error::last_os_error().to_string())
        } else {
            Ok(())
        }
    }

    pub fn new(proc_root: Option<&str>) -> Result<Self, String> {
        let c_root = match proc_root {
            Some(p) => Some(CString::new(p).map_err(|_| "invalid proc root".to_string())?),
            None => None,
        };
        let ptr = c_root
            .as_ref()
            .map(|c| c.as_ptr())
            .unwrap_or(std::ptr::null());

        let handle = unsafe { ffi::cow_monitor_new(ptr) };
        if handle.is_null() {
            return Err("failed to allocate monitor".to_string());
        }
        Ok(Monitor { handle })
    }

    pub fn sample(&mut self, top_count: usize) -> Result<Snapshot, String> {
        let mut raw: Box<MaybeUninit<ffi::CowSample>> = Box::new(MaybeUninit::uninit());
        let mut err = [0 as c_char; ffi::COW_NAME * 4];

        let rc = unsafe {
            ffi::cow_monitor_sample(
                self.handle,
                top_count,
                raw.as_mut_ptr(),
                err.as_mut_ptr(),
                err.len(),
            )
        };

        if rc != 0 {
            return Err(c_str(&err));
        }

        let raw = unsafe { raw.assume_init() };
        Ok(convert(&raw))
    }
}

impl Drop for Monitor {
    fn drop(&mut self) {
        unsafe { ffi::cow_monitor_free(self.handle) };
    }
}

fn convert(s: &ffi::CowSample) -> Snapshot {
    let cores = s.cpu.cores[..(s.cpu.core_count.max(0) as usize).min(ffi::COW_MAX_CORES)].to_vec();

    let ifaces = s.net.ifaces[..(s.net.iface_count.max(0) as usize).min(ffi::COW_MAX_IFACES)]
        .iter()
        .map(|i| Iface {
            name: c_str(&i.name),
            rx_bytes: i.rx_bytes,
            tx_bytes: i.tx_bytes,
            rx_bps: i.rx_bps,
            tx_bps: i.tx_bps,
        })
        .collect();

    let conns = s.conns[..(s.conn_count.max(0) as usize).min(ffi::COW_MAX_CONNS)]
        .iter()
        .map(|c| Conn {
            proto: c_str(&c.proto),
            local: c_str(&c.local),
            remote: c_str(&c.remote),
            state: c_str(&c.state),
            uid: c.uid,
        })
        .collect();

    let mounts = s.mounts[..(s.mount_count.max(0) as usize).min(ffi::COW_MAX_MOUNTS)]
        .iter()
        .map(|m| Mount {
            source: c_str(&m.source),
            mount: c_str(&m.mount),
            fstype: c_str(&m.fstype),
            total_kb: m.total_kb,
            used_kb: m.used_kb,
            avail_kb: m.avail_kb,
            used_percent: m.used_percent,
        })
        .collect();

    let map_procs = |arr: &[ffi::CowProc], count: i32| -> Vec<Proc> {
        arr[..(count.max(0) as usize).min(ffi::COW_MAX_PROCS)]
            .iter()
            .map(|p| Proc {
                pid: p.pid,
                ppid: p.ppid,
                name: c_str(&p.name),
                state: (p.state as u8) as char,
                threads: p.threads,
                uid: p.uid,
                cpu_percent: p.cpu_percent,
                rss_kb: p.rss_kb,
            })
            .collect()
    };

    let cpu_freqs = s.cpu_freqs[..(s.cpu_freq_count.max(0) as usize).min(ffi::COW_MAX_CORES)]
        .iter()
        .map(|f| CpuFreq {
            core_id: f.core_id,
            freq_mhz: f.freq_mhz,
        })
        .collect();

    let thermals = s.thermals[..(s.thermal_count.max(0) as usize).min(ffi::COW_MAX_THERMAL)]
        .iter()
        .map(|t| Thermal {
            name: c_str(&t.name),
            temp_c: t.temp_c,
        })
        .collect();

    Snapshot {
        cpu: Cpu {
            total_percent: s.cpu.total_percent,
            cores,
            load1: s.cpu.load1,
            load5: s.cpu.load5,
            load15: s.cpu.load15,
            uptime_seconds: s.cpu.uptime_seconds,
        },
        mem: Mem {
            total_kb: s.mem.total_kb,
            used_kb: s.mem.used_kb,
            available_kb: s.mem.available_kb,
            buffers_kb: s.mem.buffers_kb,
            cached_kb: s.mem.cached_kb,
            swap_total_kb: s.mem.swap_total_kb,
            swap_used_kb: s.mem.swap_used_kb,
        },
        net: Net {
            ifaces,
            total_rx_bps: s.net.total_rx_bps,
            total_tx_bps: s.net.total_tx_bps,
            tcp_estab: s.net.tcp_estab,
            tcp_listen: s.net.tcp_listen,
            tcp_time_wait: s.net.tcp_time_wait,
            tcp_other: s.net.tcp_other,
            udp_count: s.net.udp_count,
        },
        disk_read_bps: s.disk_read_bps,
        disk_write_bps: s.disk_write_bps,
        mounts,
        conns,
        top_cpu: map_procs(&s.top_cpu, s.top_cpu_count),
        top_mem: map_procs(&s.top_mem, s.top_mem_count),
        proc_total: s.proc_total,
        hostname: c_str(&s.hostname),
        kernel: c_str(&s.kernel),
        ctx_switches: s.ctx_switches,
        interrupts: s.interrupts,
        cpu_freqs,
        thermals,
    }
}
