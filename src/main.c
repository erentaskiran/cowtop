#define _POSIX_C_SOURCE 200809L

#include "proc_reader.h"

#include <errno.h>
#include <pthread.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define DEFAULT_INTERVAL_SECONDS 2U
#define DEFAULT_TOP_COUNT 5U
#define INITIAL_SAMPLE_DELAY_MS 250U
#define WAIT_POLL_MS 200L
#define MAX_TOP_COUNT 1024U

typedef struct {
    int watch;
    unsigned int interval_seconds;
    size_t top_count;
    const char *output_path;
    const char *proc_root;
} Options;

typedef struct {
    pthread_mutex_t mutex;
    pthread_cond_t cond;
    ProcSnapshot snapshot;
    int has_snapshot;
    unsigned long sequence;
    int stop;
    int fatal;
    char error[PROC_READER_ERROR_LEN];
} SharedState;

typedef struct {
    SharedState *shared;
    const char *proc_root;
    unsigned int interval_seconds;
    size_t top_count;
} CollectorArgs;

static volatile sig_atomic_t g_stop_requested = 0;

static void handle_sigint(int signo)
{
    (void)signo;
    g_stop_requested = 1;
}

static void print_usage(FILE *out)
{
    fprintf(out,
            "Usage: cowtop [options]\n"
            "\n"
            "Options:\n"
            "  -w                  refresh continuously\n"
            "  -i SECONDS          refresh interval, default 2\n"
            "  -n COUNT            number of top processes, default 5\n"
            "  -o PATH             write the latest snapshot report to PATH\n"
            "  --proc-root PATH    proc filesystem root, default /proc\n"
            "  -h, --help          show this help\n");
}

static int parse_positive_uint(const char *text,
                               unsigned int min_value,
                               unsigned int max_value,
                               unsigned int *value)
{
    char *end = NULL;
    unsigned long parsed;

    errno = 0;
    parsed = strtoul(text, &end, 10);
    if (errno != 0 || end == text || *end != '\0' || parsed < min_value || parsed > max_value) {
        return -1;
    }

    *value = (unsigned int)parsed;
    return 0;
}

static int parse_options(int argc, char **argv, Options *options)
{
    int i;

    options->watch = 0;
    options->interval_seconds = DEFAULT_INTERVAL_SECONDS;
    options->top_count = DEFAULT_TOP_COUNT;
    options->output_path = NULL;
    options->proc_root = "/proc";

    for (i = 1; i < argc; i++) {
        const char *arg = argv[i];

        if (strcmp(arg, "-w") == 0) {
            options->watch = 1;
        } else if (strcmp(arg, "-h") == 0 || strcmp(arg, "--help") == 0) {
            print_usage(stdout);
            exit(0);
        } else if (strcmp(arg, "-i") == 0 || strncmp(arg, "-i", 2) == 0) {
            const char *value = NULL;
            unsigned int parsed = 0;

            if (strcmp(arg, "-i") == 0) {
                if (i + 1 >= argc) {
                    fprintf(stderr, "cowtop: -i requires a value\n");
                    return -1;
                }
                value = argv[++i];
            } else {
                value = arg + 2;
            }

            if (parse_positive_uint(value, 1U, 86400U, &parsed) != 0) {
                fprintf(stderr, "cowtop: invalid interval: %s\n", value);
                return -1;
            }
            options->interval_seconds = parsed;
        } else if (strcmp(arg, "-n") == 0 || strncmp(arg, "-n", 2) == 0) {
            const char *value = NULL;
            unsigned int parsed = 0;

            if (strcmp(arg, "-n") == 0) {
                if (i + 1 >= argc) {
                    fprintf(stderr, "cowtop: -n requires a value\n");
                    return -1;
                }
                value = argv[++i];
            } else {
                value = arg + 2;
            }

            if (parse_positive_uint(value, 1U, MAX_TOP_COUNT, &parsed) != 0) {
                fprintf(stderr, "cowtop: invalid top process count: %s\n", value);
                return -1;
            }
            options->top_count = parsed;
        } else if (strcmp(arg, "-o") == 0 || strncmp(arg, "-o", 2) == 0) {
            if (strcmp(arg, "-o") == 0) {
                if (i + 1 >= argc) {
                    fprintf(stderr, "cowtop: -o requires a path\n");
                    return -1;
                }
                options->output_path = argv[++i];
            } else {
                options->output_path = arg + 2;
                if (*options->output_path == '\0') {
                    fprintf(stderr, "cowtop: -o requires a path\n");
                    return -1;
                }
            }
        } else if (strcmp(arg, "--proc-root") == 0) {
            if (i + 1 >= argc) {
                fprintf(stderr, "cowtop: --proc-root requires a path\n");
                return -1;
            }
            options->proc_root = argv[++i];
        } else if (strncmp(arg, "--proc-root=", 12) == 0) {
            options->proc_root = arg + 12;
            if (*options->proc_root == '\0') {
                fprintf(stderr, "cowtop: --proc-root requires a path\n");
                return -1;
            }
        } else {
            fprintf(stderr, "cowtop: unknown option: %s\n", arg);
            return -1;
        }
    }

    return 0;
}

static int shared_state_init(SharedState *shared)
{
    int rc;

    memset(shared, 0, sizeof(*shared));
    proc_snapshot_init(&shared->snapshot);

    rc = pthread_mutex_init(&shared->mutex, NULL);
    if (rc != 0) {
        fprintf(stderr, "cowtop: pthread_mutex_init failed: %s\n", strerror(rc));
        return -1;
    }

    rc = pthread_cond_init(&shared->cond, NULL);
    if (rc != 0) {
        pthread_mutex_destroy(&shared->mutex);
        fprintf(stderr, "cowtop: pthread_cond_init failed: %s\n", strerror(rc));
        return -1;
    }

    return 0;
}

static void shared_state_destroy(SharedState *shared)
{
    proc_snapshot_free(&shared->snapshot);
    pthread_cond_destroy(&shared->cond);
    pthread_mutex_destroy(&shared->mutex);
}

static void deadline_after_ms(struct timespec *deadline, long milliseconds)
{
    clock_gettime(CLOCK_REALTIME, deadline);
    deadline->tv_sec += milliseconds / 1000L;
    deadline->tv_nsec += (milliseconds % 1000L) * 1000000L;
    if (deadline->tv_nsec >= 1000000000L) {
        deadline->tv_sec++;
        deadline->tv_nsec -= 1000000000L;
    }
}

static int shared_should_stop(SharedState *shared)
{
    int stop;

    pthread_mutex_lock(&shared->mutex);
    stop = shared->stop;
    pthread_mutex_unlock(&shared->mutex);

    return stop || g_stop_requested;
}

static void request_stop(SharedState *shared)
{
    pthread_mutex_lock(&shared->mutex);
    shared->stop = 1;
    pthread_cond_broadcast(&shared->cond);
    pthread_mutex_unlock(&shared->mutex);
}

static int sleep_interruptible(SharedState *shared, unsigned int milliseconds)
{
    unsigned int elapsed = 0;

    while (elapsed < milliseconds) {
        unsigned int chunk = milliseconds - elapsed;
        struct timespec delay;

        if (shared_should_stop(shared)) {
            return 1;
        }

        if (chunk > 100U) {
            chunk = 100U;
        }

        delay.tv_sec = (time_t)(chunk / 1000U);
        delay.tv_nsec = (long)(chunk % 1000U) * 1000000L;
        while (nanosleep(&delay, &delay) != 0 && errno == EINTR) {
            if (shared_should_stop(shared)) {
                return 1;
            }
        }
        elapsed += chunk;
    }

    return shared_should_stop(shared);
}

static void publish_fatal(SharedState *shared, const char *message)
{
    pthread_mutex_lock(&shared->mutex);
    shared->fatal = 1;
    snprintf(shared->error, sizeof(shared->error), "%s", message);
    pthread_cond_broadcast(&shared->cond);
    pthread_mutex_unlock(&shared->mutex);
}

static void publish_snapshot(SharedState *shared, ProcSnapshot *snapshot)
{
    pthread_mutex_lock(&shared->mutex);
    proc_snapshot_free(&shared->snapshot);
    shared->snapshot = *snapshot;
    proc_snapshot_init(snapshot);
    shared->has_snapshot = 1;
    shared->sequence++;
    pthread_cond_broadcast(&shared->cond);
    pthread_mutex_unlock(&shared->mutex);
}

static void *collector_main(void *arg)
{
    CollectorArgs *collector = (CollectorArgs *)arg;
    ProcSample previous;
    ProcSample current;
    ProcSnapshot snapshot;
    char error[PROC_READER_ERROR_LEN];
    int first_snapshot = 1;

    proc_sample_init(&previous);
    proc_sample_init(&current);
    proc_snapshot_init(&snapshot);

    if (proc_read_sample(collector->proc_root, &previous, error, sizeof(error)) != 0) {
        publish_fatal(collector->shared, error);
        proc_sample_free(&previous);
        return NULL;
    }

    while (!shared_should_stop(collector->shared)) {
        unsigned int delay_ms = first_snapshot
                                    ? INITIAL_SAMPLE_DELAY_MS
                                    : collector->interval_seconds * 1000U;

        if (sleep_interruptible(collector->shared, delay_ms)) {
            break;
        }

        if (proc_read_sample(collector->proc_root, &current, error, sizeof(error)) != 0) {
            publish_fatal(collector->shared, error);
            break;
        }

        if (proc_build_snapshot(&previous,
                                &current,
                                collector->top_count,
                                &snapshot,
                                error,
                                sizeof(error)) != 0) {
            publish_fatal(collector->shared, error);
            break;
        }

        publish_snapshot(collector->shared, &snapshot);
        proc_sample_free(&previous);
        previous = current;
        proc_sample_init(&current);
        first_snapshot = 0;
    }

    proc_snapshot_free(&snapshot);
    proc_sample_free(&current);
    proc_sample_free(&previous);
    return NULL;
}

static int copy_next_snapshot(SharedState *shared,
                              ProcSnapshot *dest,
                              unsigned long *last_sequence,
                              int require_new)
{
    int result = 0;

    pthread_mutex_lock(&shared->mutex);
    while (!g_stop_requested && !shared->fatal &&
           (!shared->has_snapshot ||
            (require_new && shared->sequence == *last_sequence))) {
        struct timespec deadline;
        deadline_after_ms(&deadline, WAIT_POLL_MS);
        pthread_cond_timedwait(&shared->cond, &shared->mutex, &deadline);
    }

    if (shared->fatal) {
        result = -1;
    } else if (shared->has_snapshot &&
               (!require_new || shared->sequence != *last_sequence)) {
        if (proc_snapshot_copy(dest, &shared->snapshot) != 0) {
            result = -2;
        } else {
            *last_sequence = shared->sequence;
            result = 1;
        }
    }
    pthread_mutex_unlock(&shared->mutex);

    return result;
}

static void format_time(time_t timestamp, char *buffer, size_t buffer_size)
{
    struct tm local_time;

    if (localtime_r(&timestamp, &local_time) == NULL ||
        strftime(buffer, buffer_size, "%Y-%m-%d %H:%M:%S %Z", &local_time) == 0) {
        snprintf(buffer, buffer_size, "unknown time");
    }
}

static void print_process_table(FILE *out,
                                const char *title,
                                const ProcProcessView *processes,
                                size_t count)
{
    size_t i;

    fprintf(out, "\n%s\n", title);
    fprintf(out, "%-8s %8s %10s  %s\n", "PID", "CPU%", "RSS MB", "COMMAND");
    fprintf(out, "%-8s %8s %10s  %s\n", "--------", "--------", "----------", "------------------------------");

    if (count == 0) {
        fprintf(out, "(no processes)\n");
        return;
    }

    for (i = 0; i < count; i++) {
        fprintf(out,
                "%-8d %8.2f %10.1f  %.48s\n",
                processes[i].pid,
                processes[i].cpu_percent,
                (double)processes[i].rss_kb / 1024.0,
                processes[i].name);
    }
}

static void print_snapshot(FILE *out, const ProcSnapshot *snapshot)
{
    char time_buffer[64];
    double memory_percent = 0.0;

    format_time(snapshot->timestamp, time_buffer, sizeof(time_buffer));
    if (snapshot->mem.total_kb > 0) {
        memory_percent = 100.0 * (double)snapshot->mem.used_kb /
                         (double)snapshot->mem.total_kb;
    }

    fprintf(out, "cowtop snapshot: %s\n", time_buffer);
    fprintf(out, "CPU usage:       %6.2f%%\n", snapshot->cpu_percent);
    fprintf(out,
            "Memory usage:    %6.1f / %.1f MB (%5.1f%%), %.1f MB available\n",
            (double)snapshot->mem.used_kb / 1024.0,
            (double)snapshot->mem.total_kb / 1024.0,
            memory_percent,
            (double)snapshot->mem.available_kb / 1024.0);
    fprintf(out, "Processes read:  %u\n", snapshot->process_count);
    fprintf(out, "Process errors:  %u\n", snapshot->skipped_processes);

    print_process_table(out, "Top CPU processes", snapshot->top_cpu, snapshot->top_cpu_count);
    print_process_table(out, "Top memory processes", snapshot->top_mem, snapshot->top_mem_count);
}

static int write_report(const char *path, const ProcSnapshot *snapshot)
{
    FILE *file = fopen(path, "w");

    if (file == NULL) {
        fprintf(stderr, "cowtop: cannot write %s: %s\n", path, strerror(errno));
        return -1;
    }

    print_snapshot(file, snapshot);
    if (fclose(file) != 0) {
        fprintf(stderr, "cowtop: cannot close %s: %s\n", path, strerror(errno));
        return -1;
    }

    return 0;
}

static int install_signal_handler(void)
{
    struct sigaction action;

    memset(&action, 0, sizeof(action));
    action.sa_handler = handle_sigint;
    sigemptyset(&action.sa_mask);

    if (sigaction(SIGINT, &action, NULL) != 0) {
        fprintf(stderr, "cowtop: sigaction failed: %s\n", strerror(errno));
        return -1;
    }

    return 0;
}

static int run_once(const Options *options)
{
    SharedState shared;
    CollectorArgs args;
    pthread_t thread;
    ProcSnapshot snapshot;
    unsigned long sequence = 0;
    int status;
    int exit_code = 0;

    proc_snapshot_init(&snapshot);
    if (shared_state_init(&shared) != 0) {
        return 1;
    }

    args.shared = &shared;
    args.proc_root = options->proc_root;
    args.interval_seconds = options->interval_seconds;
    args.top_count = options->top_count;

    status = pthread_create(&thread, NULL, collector_main, &args);
    if (status != 0) {
        fprintf(stderr, "cowtop: pthread_create failed: %s\n", strerror(status));
        shared_state_destroy(&shared);
        return 1;
    }

    status = copy_next_snapshot(&shared, &snapshot, &sequence, 0);
    request_stop(&shared);
    pthread_join(thread, NULL);

    if (status == -1) {
        fprintf(stderr, "cowtop: %s\n", shared.error);
        exit_code = 1;
    } else if (status == -2) {
        fprintf(stderr, "cowtop: out of memory while copying snapshot\n");
        exit_code = 1;
    } else if (status == 0) {
        fprintf(stderr, "cowtop: interrupted before a snapshot was available\n");
        exit_code = 130;
    } else {
        print_snapshot(stdout, &snapshot);
        if (options->output_path != NULL && write_report(options->output_path, &snapshot) != 0) {
            exit_code = 1;
        }
    }

    proc_snapshot_free(&snapshot);
    shared_state_destroy(&shared);
    return exit_code;
}

static int run_watch(const Options *options)
{
    SharedState shared;
    CollectorArgs args;
    pthread_t thread;
    unsigned long sequence = 0;
    int create_status;
    int exit_code = 0;

    if (install_signal_handler() != 0) {
        return 1;
    }

    if (shared_state_init(&shared) != 0) {
        return 1;
    }

    args.shared = &shared;
    args.proc_root = options->proc_root;
    args.interval_seconds = options->interval_seconds;
    args.top_count = options->top_count;

    create_status = pthread_create(&thread, NULL, collector_main, &args);
    if (create_status != 0) {
        fprintf(stderr, "cowtop: pthread_create failed: %s\n", strerror(create_status));
        shared_state_destroy(&shared);
        return 1;
    }

    while (!g_stop_requested) {
        ProcSnapshot snapshot;
        int status;

        proc_snapshot_init(&snapshot);
        status = copy_next_snapshot(&shared, &snapshot, &sequence, 1);
        if (status == -1) {
            fprintf(stderr, "cowtop: %s\n", shared.error);
            exit_code = 1;
            proc_snapshot_free(&snapshot);
            break;
        }
        if (status == -2) {
            fprintf(stderr, "cowtop: out of memory while copying snapshot\n");
            exit_code = 1;
            proc_snapshot_free(&snapshot);
            break;
        }
        if (status == 0) {
            proc_snapshot_free(&snapshot);
            continue;
        }

        printf("\033[2J\033[H");
        print_snapshot(stdout, &snapshot);
        fflush(stdout);

        if (options->output_path != NULL && write_report(options->output_path, &snapshot) != 0) {
            exit_code = 1;
            proc_snapshot_free(&snapshot);
            break;
        }

        proc_snapshot_free(&snapshot);
    }

    request_stop(&shared);
    pthread_join(thread, NULL);
    shared_state_destroy(&shared);
    if (g_stop_requested && exit_code == 0) {
        fputc('\n', stdout);
    }

    return exit_code;
}

int main(int argc, char **argv)
{
    Options options;

    if (parse_options(argc, argv, &options) != 0) {
        print_usage(stderr);
        return 2;
    }

    if (options.watch) {
        return run_watch(&options);
    }

    return run_once(&options);
}
