//! Raw FFI mirror of `csrc/cowsys.h`. Field order, types and array sizes must
//! stay byte-for-byte identical to the C definitions.
#![allow(non_snake_case)]

use std::os::raw::{c_char, c_int};

pub const COW_MAX_CORES: usize = 256;
pub const COW_MAX_IFACES: usize = 32;
pub const COW_MAX_MOUNTS: usize = 64;
pub const COW_MAX_CONNS: usize = 256;
pub const COW_MAX_PROCS: usize = 64;
pub const COW_MAX_THERMAL: usize = 16;

pub const COW_NAME: usize = 64;
pub const COW_PATH: usize = 128;
pub const COW_ADDR: usize = 48;
pub const COW_STATE: usize = 16;
pub const COW_PROTO: usize = 8;
pub const COW_HOSTNAME_LEN: usize = 64;
pub const COW_KERNEL_LEN: usize = 256;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CowCpu {
    pub total_percent: f64,
    pub cores: [f64; COW_MAX_CORES],
    pub core_count: c_int,
    pub load1: f64,
    pub load5: f64,
    pub load15: f64,
    pub uptime_seconds: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CowMem {
    pub total_kb: u64,
    pub used_kb: u64,
    pub available_kb: u64,
    pub buffers_kb: u64,
    pub cached_kb: u64,
    pub swap_total_kb: u64,
    pub swap_used_kb: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CowIface {
    pub name: [c_char; COW_NAME],
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_bps: f64,
    pub tx_bps: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CowNet {
    pub ifaces: [CowIface; COW_MAX_IFACES],
    pub iface_count: c_int,
    pub total_rx_bps: f64,
    pub total_tx_bps: f64,
    pub tcp_estab: c_int,
    pub tcp_listen: c_int,
    pub tcp_time_wait: c_int,
    pub tcp_other: c_int,
    pub udp_count: c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CowConn {
    pub proto: [c_char; COW_PROTO],
    pub local: [c_char; COW_ADDR],
    pub remote: [c_char; COW_ADDR],
    pub state: [c_char; COW_STATE],
    pub uid: c_int,
    pub inode: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CowMount {
    pub source: [c_char; COW_NAME],
    pub mount: [c_char; COW_PATH],
    pub fstype: [c_char; COW_NAME],
    pub total_kb: u64,
    pub used_kb: u64,
    pub avail_kb: u64,
    pub used_percent: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CowProc {
    pub pid: c_int,
    pub ppid: c_int,
    pub name: [c_char; COW_NAME],
    pub state: c_char,
    pub threads: c_int,
    pub uid: c_int,
    pub cpu_percent: f64,
    pub rss_kb: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CowCpuFreq {
    pub core_id: c_int,
    pub freq_mhz: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CowThermal {
    pub name: [c_char; COW_NAME],
    pub temp_c: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CowSample {
    pub timestamp: i64,
    pub cpu: CowCpu,
    pub mem: CowMem,
    pub net: CowNet,
    pub disk_read_bps: f64,
    pub disk_write_bps: f64,
    pub mounts: [CowMount; COW_MAX_MOUNTS],
    pub mount_count: c_int,
    pub conns: [CowConn; COW_MAX_CONNS],
    pub conn_count: c_int,
    pub top_cpu: [CowProc; COW_MAX_PROCS],
    pub top_cpu_count: c_int,
    pub top_mem: [CowProc; COW_MAX_PROCS],
    pub top_mem_count: c_int,
    pub proc_total: u32,
    pub proc_skipped: u32,

    pub hostname: [c_char; COW_HOSTNAME_LEN],
    pub kernel: [c_char; COW_KERNEL_LEN],
    pub ctx_switches: u64,
    pub interrupts: u64,

    pub cpu_freqs: [CowCpuFreq; COW_MAX_CORES],
    pub cpu_freq_count: c_int,

    pub thermals: [CowThermal; COW_MAX_THERMAL],
    pub thermal_count: c_int,
}

#[repr(C)]
pub struct CowMonitor {
    _private: [u8; 0],
}

extern "C" {
    pub fn cow_monitor_new(proc_root: *const c_char) -> *mut CowMonitor;
    pub fn cow_monitor_free(monitor: *mut CowMonitor);
    pub fn cow_monitor_sample(
        monitor: *mut CowMonitor,
        top_count: usize,
        out: *mut CowSample,
        err: *mut c_char,
        err_len: usize,
    ) -> c_int;
}
