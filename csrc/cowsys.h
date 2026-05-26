#ifndef COWTOP_COWSYS_H
#define COWTOP_COWSYS_H

#include <stddef.h>

/* Fixed-capacity layout so the whole sample is a single flat, FFI-friendly
 * struct that the Rust side can mirror with #[repr(C)] and fill by pointer.
 * No heap ownership crosses the boundary except the opaque monitor handle. */

#define COW_MAX_CORES 256
#define COW_MAX_IFACES 32
#define COW_MAX_MOUNTS 64
#define COW_MAX_CONNS 256
#define COW_MAX_PROCS 64

#define COW_NAME 64
#define COW_PATH 128
#define COW_ADDR 48
#define COW_STATE 16
#define COW_PROTO 8

#define COW_ERR_LEN 256

typedef struct {
    double total_percent;
    double cores[COW_MAX_CORES];
    int core_count;
    double load1;
    double load5;
    double load15;
    double uptime_seconds;
} CowCpu;

typedef struct {
    unsigned long long total_kb;
    unsigned long long used_kb;
    unsigned long long available_kb;
    unsigned long long buffers_kb;
    unsigned long long cached_kb;
    unsigned long long swap_total_kb;
    unsigned long long swap_used_kb;
} CowMem;

typedef struct {
    char name[COW_NAME];
    unsigned long long rx_bytes;
    unsigned long long tx_bytes;
    double rx_bps;
    double tx_bps;
} CowIface;

typedef struct {
    CowIface ifaces[COW_MAX_IFACES];
    int iface_count;
    double total_rx_bps;
    double total_tx_bps;
    int tcp_estab;
    int tcp_listen;
    int tcp_time_wait;
    int tcp_other;
    int udp_count;
} CowNet;

typedef struct {
    char proto[COW_PROTO];
    char local[COW_ADDR];
    char remote[COW_ADDR];
    char state[COW_STATE];
    int uid;
    unsigned long inode;
} CowConn;

typedef struct {
    char source[COW_NAME];
    char mount[COW_PATH];
    char fstype[COW_NAME];
    unsigned long long total_kb;
    unsigned long long used_kb;
    unsigned long long avail_kb;
    double used_percent;
} CowMount;

typedef struct {
    int pid;
    char name[COW_NAME];
    double cpu_percent;
    unsigned long long rss_kb;
} CowProc;

typedef struct {
    long long timestamp;

    CowCpu cpu;
    CowMem mem;
    CowNet net;

    double disk_read_bps;
    double disk_write_bps;

    CowMount mounts[COW_MAX_MOUNTS];
    int mount_count;

    CowConn conns[COW_MAX_CONNS];
    int conn_count;

    CowProc top_cpu[COW_MAX_PROCS];
    int top_cpu_count;
    CowProc top_mem[COW_MAX_PROCS];
    int top_mem_count;

    unsigned int proc_total;
    unsigned int proc_skipped;
} CowSample;

typedef struct CowMonitor CowMonitor;

/* proc_root may be NULL to default to "/proc". */
CowMonitor *cow_monitor_new(const char *proc_root);
void cow_monitor_free(CowMonitor *monitor);

/* Fills *out with a fresh sample. Rates (cpu, net, disk) are zero on the very
 * first call and meaningful afterwards. top_count is clamped to COW_MAX_PROCS.
 * Returns 0 on success, -1 on error (err receives a message). */
int cow_monitor_sample(CowMonitor *monitor,
                       size_t top_count,
                       CowSample *out,
                       char *err,
                       size_t err_len);

#endif
