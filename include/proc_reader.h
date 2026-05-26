#ifndef COWTOP_PROC_READER_H
#define COWTOP_PROC_READER_H

#include <stddef.h>
#include <time.h>

#define PROC_READER_NAME_LEN 256
#define PROC_READER_ERROR_LEN 256

typedef struct {
    unsigned long long total;
    unsigned long long idle;
} ProcCpuTimes;

typedef struct {
    unsigned long long total_kb;
    unsigned long long free_kb;
    unsigned long long available_kb;
    unsigned long long used_kb;
} ProcMemInfo;

typedef struct {
    int pid;
    char name[PROC_READER_NAME_LEN];
    unsigned long long cpu_ticks;
    unsigned long rss_kb;
} ProcProcessSample;

typedef struct {
    ProcCpuTimes cpu;
    ProcMemInfo mem;
    ProcProcessSample *processes;
    size_t process_count;
    size_t process_capacity;
    unsigned int skipped_processes;
} ProcSample;

typedef struct {
    int pid;
    char name[PROC_READER_NAME_LEN];
    double cpu_percent;
    unsigned long rss_kb;
} ProcProcessView;

typedef struct {
    time_t timestamp;
    double cpu_percent;
    ProcMemInfo mem;
    unsigned int process_count;
    unsigned int skipped_processes;
    ProcProcessView *top_cpu;
    size_t top_cpu_count;
    ProcProcessView *top_mem;
    size_t top_mem_count;
} ProcSnapshot;

void proc_sample_init(ProcSample *sample);
void proc_sample_free(ProcSample *sample);

int proc_read_sample(const char *proc_root, ProcSample *sample, char *error, size_t error_size);

void proc_snapshot_init(ProcSnapshot *snapshot);
void proc_snapshot_free(ProcSnapshot *snapshot);
int proc_snapshot_copy(ProcSnapshot *dest, const ProcSnapshot *src);
int proc_build_snapshot(const ProcSample *previous,
                        const ProcSample *current,
                        size_t top_count,
                        ProcSnapshot *snapshot,
                        char *error,
                        size_t error_size);

#endif
