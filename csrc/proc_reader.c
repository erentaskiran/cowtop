#define _POSIX_C_SOURCE 200809L

#include "proc_reader.h"

#include <ctype.h>
#include <dirent.h>
#include <errno.h>
#include <limits.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifndef COWTOP_PATH_MAX
#define COWTOP_PATH_MAX 4096
#endif

static void set_error(char *error, size_t error_size, const char *format, ...)
{
    va_list args;

    if (error == NULL || error_size == 0) {
        return;
    }

    va_start(args, format);
    vsnprintf(error, error_size, format, args);
    va_end(args);
}

static int build_path(char *buffer, size_t buffer_size, const char *root, const char *name)
{
    int written = snprintf(buffer, buffer_size, "%s/%s", root, name);
    return written >= 0 && (size_t)written < buffer_size;
}

static int build_pid_path(char *buffer,
                          size_t buffer_size,
                          const char *root,
                          int pid,
                          const char *name)
{
    int written = snprintf(buffer, buffer_size, "%s/%d/%s", root, pid, name);
    return written >= 0 && (size_t)written < buffer_size;
}

static int is_pid_name(const char *name)
{
    const unsigned char *cursor = (const unsigned char *)name;

    if (*cursor == '\0') {
        return 0;
    }

    while (*cursor != '\0') {
        if (!isdigit(*cursor)) {
            return 0;
        }
        cursor++;
    }

    return 1;
}

static int parse_unsigned_long_long(const char *text, unsigned long long *value)
{
    char *end = NULL;
    unsigned long long parsed;

    errno = 0;
    parsed = strtoull(text, &end, 10);
    if (errno != 0 || end == text) {
        return -1;
    }

    *value = parsed;
    return 0;
}

static int parse_pid(const char *text, int *pid)
{
    char *end = NULL;
    long parsed;

    errno = 0;
    parsed = strtol(text, &end, 10);
    if (errno != 0 || end == text || *end != '\0' || parsed <= 0 || parsed > INT_MAX) {
        return -1;
    }

    *pid = (int)parsed;
    return 0;
}

static int append_process(ProcSample *sample, const ProcProcessSample *process)
{
    ProcProcessSample *grown;
    size_t next_capacity;

    if (sample->process_count == sample->process_capacity) {
        next_capacity = sample->process_capacity == 0 ? 64 : sample->process_capacity * 2;
        grown = realloc(sample->processes, next_capacity * sizeof(*sample->processes));
        if (grown == NULL) {
            return -1;
        }
        sample->processes = grown;
        sample->process_capacity = next_capacity;
    }

    sample->processes[sample->process_count] = *process;
    sample->process_count++;
    return 0;
}

void proc_sample_init(ProcSample *sample)
{
    if (sample == NULL) {
        return;
    }

    memset(sample, 0, sizeof(*sample));
}

void proc_sample_free(ProcSample *sample)
{
    if (sample == NULL) {
        return;
    }

    free(sample->processes);
    proc_sample_init(sample);
}

static int read_cpu_times(const char *proc_root,
                          ProcCpuTimes *times,
                          char *error,
                          size_t error_size)
{
    char path[COWTOP_PATH_MAX];
    char line[1024];
    char label[16];
    unsigned long long fields[10] = {0};
    int parsed;
    int i;
    FILE *file;

    if (!build_path(path, sizeof(path), proc_root, "stat")) {
        set_error(error, error_size, "proc path is too long: %s/stat", proc_root);
        return -1;
    }

    file = fopen(path, "r");
    if (file == NULL) {
        set_error(error, error_size, "cannot read %s: %s", path, strerror(errno));
        return -1;
    }

    if (fgets(line, sizeof(line), file) == NULL) {
        set_error(error, error_size, "cannot read first line from %s", path);
        fclose(file);
        return -1;
    }
    fclose(file);

    parsed = sscanf(line,
                    "%15s %llu %llu %llu %llu %llu %llu %llu %llu %llu %llu",
                    label,
                    &fields[0],
                    &fields[1],
                    &fields[2],
                    &fields[3],
                    &fields[4],
                    &fields[5],
                    &fields[6],
                    &fields[7],
                    &fields[8],
                    &fields[9]);
    if (parsed < 5 || strcmp(label, "cpu") != 0) {
        set_error(error, error_size, "cannot parse aggregate CPU line in %s", path);
        return -1;
    }

    times->total = 0;
    for (i = 0; i < parsed - 1; i++) {
        times->total += fields[i];
    }
    times->idle = fields[3];
    if (parsed > 5) {
        times->idle += fields[4];
    }

    return 0;
}

static int read_mem_info(const char *proc_root,
                         ProcMemInfo *mem,
                         char *error,
                         size_t error_size)
{
    char path[COWTOP_PATH_MAX];
    char line[512];
    unsigned long long buffers_kb = 0;
    unsigned long long cached_kb = 0;
    unsigned long long sreclaimable_kb = 0;
    unsigned long long shmem_kb = 0;
    unsigned long long swap_total_kb = 0;
    unsigned long long swap_free_kb = 0;
    FILE *file;

    if (!build_path(path, sizeof(path), proc_root, "meminfo")) {
        set_error(error, error_size, "proc path is too long: %s/meminfo", proc_root);
        return -1;
    }

    file = fopen(path, "r");
    if (file == NULL) {
        set_error(error, error_size, "cannot read %s: %s", path, strerror(errno));
        return -1;
    }

    memset(mem, 0, sizeof(*mem));
    while (fgets(line, sizeof(line), file) != NULL) {
        char key[64];
        unsigned long long value = 0;

        if (sscanf(line, "%63[^:]: %llu", key, &value) != 2) {
            continue;
        }

        if (strcmp(key, "MemTotal") == 0) {
            mem->total_kb = value;
        } else if (strcmp(key, "MemFree") == 0) {
            mem->free_kb = value;
        } else if (strcmp(key, "MemAvailable") == 0) {
            mem->available_kb = value;
        } else if (strcmp(key, "Buffers") == 0) {
            buffers_kb = value;
        } else if (strcmp(key, "Cached") == 0) {
            cached_kb = value;
        } else if (strcmp(key, "SReclaimable") == 0) {
            sreclaimable_kb = value;
        } else if (strcmp(key, "Shmem") == 0) {
            shmem_kb = value;
        } else if (strcmp(key, "SwapTotal") == 0) {
            swap_total_kb = value;
        } else if (strcmp(key, "SwapFree") == 0) {
            swap_free_kb = value;
        }
    }

    if (ferror(file)) {
        set_error(error, error_size, "error while reading %s", path);
        fclose(file);
        return -1;
    }
    fclose(file);

    if (mem->total_kb == 0) {
        set_error(error, error_size, "cannot parse MemTotal in %s", path);
        return -1;
    }

    if (mem->available_kb == 0) {
        unsigned long long estimated_available = mem->free_kb + buffers_kb + cached_kb + sreclaimable_kb;
        if (estimated_available > shmem_kb) {
            estimated_available -= shmem_kb;
        }
        mem->available_kb = estimated_available;
    }

    if (mem->available_kb > mem->total_kb) {
        mem->available_kb = mem->total_kb;
    }
    mem->used_kb = mem->total_kb - mem->available_kb;

    mem->buffers_kb = buffers_kb;
    mem->cached_kb = cached_kb + sreclaimable_kb;
    mem->swap_total_kb = swap_total_kb;
    mem->swap_free_kb = swap_free_kb;
    mem->swap_used_kb = swap_total_kb > swap_free_kb ? swap_total_kb - swap_free_kb : 0;

    return 0;
}

static int parse_stat_line(const char *line, ProcProcessSample *process)
{
    const char *left = strchr(line, '(');
    const char *right = strrchr(line, ')');
    char rest[4096];
    char *cursor;
    char *saveptr = NULL;
    char *token;
    unsigned long long utime = 0;
    unsigned long long stime = 0;
    int have_utime = 0;
    int have_stime = 0;
    int field = 3;
    size_t name_len;

    if (left == NULL || right == NULL || right <= left + 1) {
        return -1;
    }

    name_len = (size_t)(right - left - 1);
    if (name_len >= sizeof(process->name)) {
        name_len = sizeof(process->name) - 1;
    }
    memcpy(process->name, left + 1, name_len);
    process->name[name_len] = '\0';

    cursor = (char *)right + 1;
    while (*cursor == ' ' || *cursor == '\t') {
        cursor++;
    }

    if (strlen(cursor) >= sizeof(rest)) {
        return -1;
    }
    strcpy(rest, cursor);

    token = strtok_r(rest, " \t\r\n", &saveptr);
    while (token != NULL) {
        if (field == 14) {
            if (parse_unsigned_long_long(token, &utime) != 0) {
                return -1;
            }
            have_utime = 1;
        } else if (field == 15) {
            if (parse_unsigned_long_long(token, &stime) != 0) {
                return -1;
            }
            have_stime = 1;
            break;
        }

        field++;
        token = strtok_r(NULL, " \t\r\n", &saveptr);
    }

    if (!have_utime || !have_stime) {
        return -1;
    }

    process->cpu_ticks = utime + stime;
    return 0;
}

static int read_process_stat(const char *proc_root, int pid, ProcProcessSample *process)
{
    char path[COWTOP_PATH_MAX];
    char line[4096];
    FILE *file;

    if (!build_pid_path(path, sizeof(path), proc_root, pid, "stat")) {
        return -1;
    }

    file = fopen(path, "r");
    if (file == NULL) {
        return -1;
    }

    if (fgets(line, sizeof(line), file) == NULL) {
        fclose(file);
        return -1;
    }
    fclose(file);

    process->pid = pid;
    process->rss_kb = 0;
    return parse_stat_line(line, process);
}

static int read_process_rss(const char *proc_root, int pid, unsigned long *rss_kb)
{
    char path[COWTOP_PATH_MAX];
    char line[512];
    FILE *file;

    if (!build_pid_path(path, sizeof(path), proc_root, pid, "status")) {
        return -1;
    }

    file = fopen(path, "r");
    if (file == NULL) {
        return -1;
    }

    *rss_kb = 0;
    while (fgets(line, sizeof(line), file) != NULL) {
        unsigned long value = 0;

        if (sscanf(line, "VmRSS: %lu", &value) == 1) {
            *rss_kb = value;
            break;
        }
    }

    if (ferror(file)) {
        fclose(file);
        return -1;
    }

    fclose(file);
    return 0;
}

static int read_processes(const char *proc_root,
                          ProcSample *sample,
                          char *error,
                          size_t error_size)
{
    DIR *dir;
    struct dirent *entry;

    dir = opendir(proc_root);
    if (dir == NULL) {
        set_error(error, error_size, "cannot open %s: %s", proc_root, strerror(errno));
        return -1;
    }

    while ((entry = readdir(dir)) != NULL) {
        ProcProcessSample process;
        int pid;

        if (!is_pid_name(entry->d_name)) {
            continue;
        }

        if (parse_pid(entry->d_name, &pid) != 0) {
            sample->skipped_processes++;
            continue;
        }

        if (read_process_stat(proc_root, pid, &process) != 0 ||
            read_process_rss(proc_root, pid, &process.rss_kb) != 0) {
            sample->skipped_processes++;
            continue;
        }

        if (append_process(sample, &process) != 0) {
            closedir(dir);
            set_error(error, error_size, "out of memory while reading processes");
            return -1;
        }
    }

    if (closedir(dir) != 0) {
        set_error(error, error_size, "cannot close %s: %s", proc_root, strerror(errno));
        return -1;
    }

    return 0;
}

int proc_read_sample(const char *proc_root, ProcSample *sample, char *error, size_t error_size)
{
    ProcSample next;

    proc_sample_init(&next);

    if (read_cpu_times(proc_root, &next.cpu, error, error_size) != 0 ||
        read_mem_info(proc_root, &next.mem, error, error_size) != 0 ||
        read_processes(proc_root, &next, error, error_size) != 0) {
        proc_sample_free(&next);
        return -1;
    }

    proc_sample_free(sample);
    *sample = next;
    return 0;
}

void proc_snapshot_init(ProcSnapshot *snapshot)
{
    if (snapshot == NULL) {
        return;
    }

    memset(snapshot, 0, sizeof(*snapshot));
}

void proc_snapshot_free(ProcSnapshot *snapshot)
{
    if (snapshot == NULL) {
        return;
    }

    free(snapshot->top_cpu);
    free(snapshot->top_mem);
    proc_snapshot_init(snapshot);
}

static const ProcProcessSample *find_previous_process(const ProcSample *previous,
                                                      const ProcProcessSample *current)
{
    size_t i;

    for (i = 0; i < previous->process_count; i++) {
        const ProcProcessSample *candidate = &previous->processes[i];
        if (candidate->pid == current->pid && strcmp(candidate->name, current->name) == 0) {
            return candidate;
        }
    }

    return NULL;
}

static int compare_cpu_desc(const void *left, const void *right)
{
    const ProcProcessView *a = (const ProcProcessView *)left;
    const ProcProcessView *b = (const ProcProcessView *)right;

    if (a->cpu_percent < b->cpu_percent) {
        return 1;
    }
    if (a->cpu_percent > b->cpu_percent) {
        return -1;
    }
    if (a->rss_kb < b->rss_kb) {
        return 1;
    }
    if (a->rss_kb > b->rss_kb) {
        return -1;
    }
    return a->pid - b->pid;
}

static int compare_mem_desc(const void *left, const void *right)
{
    const ProcProcessView *a = (const ProcProcessView *)left;
    const ProcProcessView *b = (const ProcProcessView *)right;

    if (a->rss_kb < b->rss_kb) {
        return 1;
    }
    if (a->rss_kb > b->rss_kb) {
        return -1;
    }
    if (a->cpu_percent < b->cpu_percent) {
        return 1;
    }
    if (a->cpu_percent > b->cpu_percent) {
        return -1;
    }
    return a->pid - b->pid;
}

int proc_snapshot_copy(ProcSnapshot *dest, const ProcSnapshot *src)
{
    ProcSnapshot copy;

    proc_snapshot_init(&copy);
    copy.timestamp = src->timestamp;
    copy.cpu_percent = src->cpu_percent;
    copy.mem = src->mem;
    copy.process_count = src->process_count;
    copy.skipped_processes = src->skipped_processes;
    copy.top_cpu_count = src->top_cpu_count;
    copy.top_mem_count = src->top_mem_count;

    if (src->top_cpu_count > 0) {
        copy.top_cpu = malloc(src->top_cpu_count * sizeof(*copy.top_cpu));
        if (copy.top_cpu == NULL) {
            proc_snapshot_free(&copy);
            return -1;
        }
        memcpy(copy.top_cpu, src->top_cpu, src->top_cpu_count * sizeof(*copy.top_cpu));
    }

    if (src->top_mem_count > 0) {
        copy.top_mem = malloc(src->top_mem_count * sizeof(*copy.top_mem));
        if (copy.top_mem == NULL) {
            proc_snapshot_free(&copy);
            return -1;
        }
        memcpy(copy.top_mem, src->top_mem, src->top_mem_count * sizeof(*copy.top_mem));
    }

    proc_snapshot_free(dest);
    *dest = copy;
    return 0;
}

int proc_build_snapshot(const ProcSample *previous,
                        const ProcSample *current,
                        size_t top_count,
                        ProcSnapshot *snapshot,
                        char *error,
                        size_t error_size)
{
    ProcSnapshot next;
    ProcProcessView *views = NULL;
    unsigned long long total_delta = 0;
    unsigned long long idle_delta = 0;
    size_t selected_count;
    size_t i;

    proc_snapshot_init(&next);

    if (current->cpu.total >= previous->cpu.total) {
        total_delta = current->cpu.total - previous->cpu.total;
    }
    if (current->cpu.idle >= previous->cpu.idle) {
        idle_delta = current->cpu.idle - previous->cpu.idle;
    }

    next.timestamp = time(NULL);
    next.mem = current->mem;
    next.process_count = (unsigned int)current->process_count;
    next.skipped_processes = current->skipped_processes;
    if (total_delta > 0 && idle_delta <= total_delta) {
        next.cpu_percent = 100.0 * (double)(total_delta - idle_delta) / (double)total_delta;
    }

    if (current->process_count > 0) {
        views = calloc(current->process_count, sizeof(*views));
        if (views == NULL) {
            set_error(error, error_size, "out of memory while building snapshot");
            return -1;
        }
    }

    for (i = 0; i < current->process_count; i++) {
        const ProcProcessSample *process = &current->processes[i];
        const ProcProcessSample *old = find_previous_process(previous, process);
        ProcProcessView *view = &views[i];

        view->pid = process->pid;
        strncpy(view->name, process->name, sizeof(view->name) - 1);
        view->name[sizeof(view->name) - 1] = '\0';
        view->rss_kb = process->rss_kb;

        if (old != NULL && process->cpu_ticks >= old->cpu_ticks && total_delta > 0) {
            unsigned long long process_delta = process->cpu_ticks - old->cpu_ticks;
            view->cpu_percent = 100.0 * (double)process_delta / (double)total_delta;
        }
    }

    selected_count = top_count;
    if (selected_count > current->process_count) {
        selected_count = current->process_count;
    }

    if (selected_count > 0) {
        next.top_cpu = malloc(selected_count * sizeof(*next.top_cpu));
        next.top_mem = malloc(selected_count * sizeof(*next.top_mem));
        if (next.top_cpu == NULL || next.top_mem == NULL) {
            free(views);
            proc_snapshot_free(&next);
            set_error(error, error_size, "out of memory while selecting top processes");
            return -1;
        }
    }

    if (selected_count > 0) {
        qsort(views, current->process_count, sizeof(*views), compare_cpu_desc);
        memcpy(next.top_cpu, views, selected_count * sizeof(*next.top_cpu));
        next.top_cpu_count = selected_count;

        qsort(views, current->process_count, sizeof(*views), compare_mem_desc);
        memcpy(next.top_mem, views, selected_count * sizeof(*next.top_mem));
        next.top_mem_count = selected_count;
    }

    free(views);
    proc_snapshot_free(snapshot);
    *snapshot = next;
    return 0;
}
