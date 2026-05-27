#define _POSIX_C_SOURCE 200809L

#include "cowsys.h"
#include "proc_reader.h"
#include "cow_net.h"
#include "cow_disk.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#ifndef COWTOP_PATH_MAX
#define COWTOP_PATH_MAX 4096
#endif

#define SECTOR_BYTES 512.0

struct CowMonitor {
    char proc_root[COWTOP_PATH_MAX];
    int has_previous;

    ProcSample prev_proc;

    unsigned long long prev_core_total[COW_MAX_CORES];
    unsigned long long prev_core_idle[COW_MAX_CORES];
    int prev_core_count;

    CowIface prev_ifaces[COW_MAX_IFACES];
    int prev_iface_count;

    unsigned long long prev_read_sectors;
    unsigned long long prev_write_sectors;

    struct timespec prev_time;
};

static void set_err(char *err, size_t err_len, const char *msg)
{
    if (err != NULL && err_len > 0) {
        snprintf(err, err_len, "%s", msg);
    }
}

CowMonitor *cow_monitor_new(const char *proc_root)
{
    CowMonitor *monitor = calloc(1, sizeof(*monitor));

    if (monitor == NULL) {
        return NULL;
    }

    snprintf(monitor->proc_root, sizeof(monitor->proc_root), "%s",
             (proc_root != NULL && proc_root[0] != '\0') ? proc_root : "/proc");
    proc_sample_init(&monitor->prev_proc);
    monitor->has_previous = 0;
    return monitor;
}

void cow_monitor_free(CowMonitor *monitor)
{
    if (monitor == NULL) {
        return;
    }
    proc_sample_free(&monitor->prev_proc);
    free(monitor);
}

static int read_core_times(const char *proc_root,
                           unsigned long long *total,
                           unsigned long long *idle,
                           int *count)
{
    char path[COWTOP_PATH_MAX];
    char line[1024];
    FILE *file;
    int n = 0;

    *count = 0;

    if (snprintf(path, sizeof(path), "%s/stat", proc_root) >= (int)sizeof(path)) {
        return -1;
    }

    file = fopen(path, "r");
    if (file == NULL) {
        return -1;
    }

    while (fgets(line, sizeof(line), file) != NULL && n < COW_MAX_CORES) {
        char label[16];
        unsigned long long fields[10] = {0};
        int index;
        int parsed;
        unsigned long long sum = 0;

        if (strncmp(line, "cpu", 3) != 0) {
            continue;
        }

        parsed = sscanf(line,
                        "%15s %llu %llu %llu %llu %llu %llu %llu %llu %llu %llu",
                        label, &fields[0], &fields[1], &fields[2], &fields[3],
                        &fields[4], &fields[5], &fields[6], &fields[7],
                        &fields[8], &fields[9]);

        /* Only per-core lines: "cpu0", "cpu1", ... (skip the aggregate "cpu"). */
        if (parsed < 5 || label[3] == '\0') {
            continue;
        }

        for (index = 0; index < parsed - 1; index++) {
            sum += fields[index];
        }
        total[n] = sum;
        idle[n] = fields[3] + (parsed > 5 ? fields[4] : 0);
        n++;
    }

    fclose(file);
    *count = n;
    return 0;
}

static int read_load_uptime(const char *proc_root, CowCpu *cpu)
{
    char path[COWTOP_PATH_MAX];
    FILE *file;

    cpu->load1 = cpu->load5 = cpu->load15 = 0.0;
    cpu->uptime_seconds = 0.0;

    if (snprintf(path, sizeof(path), "%s/loadavg", proc_root) < (int)sizeof(path)) {
        file = fopen(path, "r");
        if (file != NULL) {
            if (fscanf(file, "%lf %lf %lf", &cpu->load1, &cpu->load5, &cpu->load15) != 3) {
                cpu->load1 = cpu->load5 = cpu->load15 = 0.0;
            }
            fclose(file);
        }
    }

    if (snprintf(path, sizeof(path), "%s/uptime", proc_root) < (int)sizeof(path)) {
        file = fopen(path, "r");
        if (file != NULL) {
            if (fscanf(file, "%lf", &cpu->uptime_seconds) != 1) {
                cpu->uptime_seconds = 0.0;
            }
            fclose(file);
        }
    }

    return 0;
}

static double timespec_delta(const struct timespec *now, const struct timespec *prev)
{
    return (double)(now->tv_sec - prev->tv_sec) +
           (double)(now->tv_nsec - prev->tv_nsec) / 1e9;
}

static void copy_proc_views(CowProc *dest, int *dest_count,
                            const ProcProcessView *src, size_t src_count)
{
    size_t i;
    size_t n = src_count > COW_MAX_PROCS ? (size_t)COW_MAX_PROCS : src_count;

    for (i = 0; i < n; i++) {
        dest[i].pid = src[i].pid;
        snprintf(dest[i].name, sizeof(dest[i].name), "%s", src[i].name);
        dest[i].cpu_percent = src[i].cpu_percent;
        dest[i].rss_kb = src[i].rss_kb;
    }
    *dest_count = (int)n;
}

int cow_monitor_sample(CowMonitor *monitor,
                       size_t top_count,
                       CowSample *out,
                       char *err,
                       size_t err_len)
{
    ProcSample current;
    ProcSnapshot snapshot;
    char inner_err[PROC_READER_ERROR_LEN];
    struct timespec now;
    double dt;
    int first;
    int core_count;
    unsigned long long core_total[COW_MAX_CORES];
    unsigned long long core_idle[COW_MAX_CORES];
    unsigned long long read_sectors = 0;
    unsigned long long write_sectors = 0;
    int i;

    if (monitor == NULL || out == NULL) {
        set_err(err, err_len, "null argument");
        return -1;
    }

    if (top_count == 0 || top_count > COW_MAX_PROCS) {
        top_count = COW_MAX_PROCS;
    }

    proc_sample_init(&current);
    proc_snapshot_init(&snapshot);
    memset(out, 0, sizeof(*out));

    if (proc_read_sample(monitor->proc_root, &current, inner_err, sizeof(inner_err)) != 0) {
        set_err(err, err_len, inner_err);
        proc_sample_free(&current);
        return -1;
    }

    first = !monitor->has_previous;
    clock_gettime(CLOCK_MONOTONIC, &now);
    dt = first ? 0.0 : timespec_delta(&now, &monitor->prev_time);
    if (dt <= 0.0) {
        dt = first ? 0.0 : 1e-3;
    }

    /* Build top-N process views (uses previous sample for CPU deltas). */
    if (proc_build_snapshot(&monitor->prev_proc, &current, top_count,
                            &snapshot, inner_err, sizeof(inner_err)) != 0) {
        set_err(err, err_len, inner_err);
        proc_snapshot_free(&snapshot);
        proc_sample_free(&current);
        return -1;
    }

    out->timestamp = (long long)time(NULL);

    /* Hostname from /proc/sys/kernel/hostname */
    {
        char hpath[COWTOP_PATH_MAX];
        FILE *hf;
        snprintf(hpath, sizeof(hpath), "%s/sys/kernel/hostname", monitor->proc_root);
        hf = fopen(hpath, "r");
        if (hf != NULL) {
            char buf[COW_HOSTNAME_LEN];
            if (fgets(buf, sizeof(buf), hf) != NULL) {
                size_t hlen = strlen(buf);
                while (hlen > 0 && (buf[hlen - 1] == '\n' || buf[hlen - 1] == '\r'))
                    buf[--hlen] = '\0';
                snprintf(out->hostname, sizeof(out->hostname), "%s", buf);
            }
            fclose(hf);
        }
    }

    /* Kernel version from /proc/version (trim after first '(') */
    {
        char kpath[COWTOP_PATH_MAX];
        FILE *kf;
        snprintf(kpath, sizeof(kpath), "%s/version", monitor->proc_root);
        kf = fopen(kpath, "r");
        if (kf != NULL) {
            char buf[COW_KERNEL_LEN];
            if (fgets(buf, sizeof(buf), kf) != NULL) {
                char *paren = strchr(buf, '(');
                size_t klen;
                if (paren != NULL) *paren = '\0';
                klen = strlen(buf);
                while (klen > 0 && (buf[klen - 1] == ' ' || buf[klen - 1] == '\n'))
                    buf[--klen] = '\0';
                snprintf(out->kernel, sizeof(out->kernel), "%s", buf);
            }
            fclose(kf);
        }
    }

    /* Context switches + total interrupts from /proc/stat */
    {
        char spath[COWTOP_PATH_MAX];
        FILE *sf;
        char sline[256];
        snprintf(spath, sizeof(spath), "%s/stat", monitor->proc_root);
        sf = fopen(spath, "r");
        if (sf != NULL) {
            while (fgets(sline, sizeof(sline), sf) != NULL) {
                unsigned long long val;
                if (strncmp(sline, "ctxt ", 5) == 0) {
                    if (sscanf(sline + 5, "%llu", &val) == 1)
                        out->ctx_switches = val;
                } else if (strncmp(sline, "intr ", 5) == 0) {
                    if (sscanf(sline + 5, "%llu", &val) == 1)
                        out->interrupts = val;
                }
            }
            fclose(sf);
        }
    }

    /* Memory. */
    out->mem.total_kb = current.mem.total_kb;
    out->mem.used_kb = current.mem.used_kb;
    out->mem.available_kb = current.mem.available_kb;
    out->mem.buffers_kb = current.mem.buffers_kb;
    out->mem.cached_kb = current.mem.cached_kb;
    out->mem.swap_total_kb = current.mem.swap_total_kb;
    out->mem.swap_used_kb = current.mem.swap_used_kb;

    /* CPU aggregate + load + uptime. */
    out->cpu.total_percent = first ? 0.0 : snapshot.cpu_percent;
    read_load_uptime(monitor->proc_root, &out->cpu);

    /* Per-core CPU. */
    read_core_times(monitor->proc_root, core_total, core_idle, &core_count);
    out->cpu.core_count = core_count;
    for (i = 0; i < core_count; i++) {
        double percent = 0.0;
        if (!first && i < monitor->prev_core_count) {
            unsigned long long td = core_total[i] >= monitor->prev_core_total[i]
                ? core_total[i] - monitor->prev_core_total[i] : 0;
            unsigned long long idd = core_idle[i] >= monitor->prev_core_idle[i]
                ? core_idle[i] - monitor->prev_core_idle[i] : 0;
            if (td > 0 && idd <= td) {
                percent = 100.0 * (double)(td - idd) / (double)td;
            }
        }
        out->cpu.cores[i] = percent;
    }

    /* Process tables. */
    copy_proc_views(out->top_cpu, &out->top_cpu_count,
                    snapshot.top_cpu, snapshot.top_cpu_count);
    copy_proc_views(out->top_mem, &out->top_mem_count,
                    snapshot.top_mem, snapshot.top_mem_count);
    out->proc_total = snapshot.process_count;
    out->proc_skipped = snapshot.skipped_processes;

    /* Network interfaces + rates. */
    cow_net_read_ifaces(monitor->proc_root, out->net.ifaces, COW_MAX_IFACES,
                        &out->net.iface_count);
    out->net.total_rx_bps = 0.0;
    out->net.total_tx_bps = 0.0;
    for (i = 0; i < out->net.iface_count; i++) {
        CowIface *cur = &out->net.ifaces[i];
        if (!first) {
            int j;
            for (j = 0; j < monitor->prev_iface_count; j++) {
                if (strcmp(cur->name, monitor->prev_ifaces[j].name) == 0) {
                    if (cur->rx_bytes >= monitor->prev_ifaces[j].rx_bytes) {
                        cur->rx_bps = (double)(cur->rx_bytes - monitor->prev_ifaces[j].rx_bytes) / dt;
                    }
                    if (cur->tx_bytes >= monitor->prev_ifaces[j].tx_bytes) {
                        cur->tx_bps = (double)(cur->tx_bytes - monitor->prev_ifaces[j].tx_bytes) / dt;
                    }
                    break;
                }
            }
        }
        if (strcmp(cur->name, "lo") != 0) {
            out->net.total_rx_bps += cur->rx_bps;
            out->net.total_tx_bps += cur->tx_bps;
        }
    }

    /* Connections (packet tracing). */
    cow_net_read_conns(monitor->proc_root, &out->net, out->conns, COW_MAX_CONNS,
                       &out->conn_count);

    /* Storage. */
    cow_disk_read_mounts(monitor->proc_root, out->mounts, COW_MAX_MOUNTS, &out->mount_count);
    cow_disk_read_io(monitor->proc_root, &read_sectors, &write_sectors);
    if (!first) {
        if (read_sectors >= monitor->prev_read_sectors) {
            out->disk_read_bps = (double)(read_sectors - monitor->prev_read_sectors) * SECTOR_BYTES / dt;
        }
        if (write_sectors >= monitor->prev_write_sectors) {
            out->disk_write_bps = (double)(write_sectors - monitor->prev_write_sectors) * SECTOR_BYTES / dt;
        }
    }

    /* Persist this sample as the baseline for the next call. */
    proc_sample_free(&monitor->prev_proc);
    monitor->prev_proc = current;          /* take ownership */
    monitor->prev_core_count = core_count;
    for (i = 0; i < core_count; i++) {
        monitor->prev_core_total[i] = core_total[i];
        monitor->prev_core_idle[i] = core_idle[i];
    }
    monitor->prev_iface_count = out->net.iface_count;
    for (i = 0; i < out->net.iface_count; i++) {
        monitor->prev_ifaces[i] = out->net.ifaces[i];
    }
    monitor->prev_read_sectors = read_sectors;
    monitor->prev_write_sectors = write_sectors;
    monitor->prev_time = now;
    monitor->has_previous = 1;

    proc_snapshot_free(&snapshot);
    return 0;
}
