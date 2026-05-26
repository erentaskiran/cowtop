#define _POSIX_C_SOURCE 200809L

#include "cow_disk.h"

#include <ctype.h>
#include <stdio.h>
#include <string.h>
#include <sys/statvfs.h>

#ifndef COWTOP_PATH_MAX
#define COWTOP_PATH_MAX 4096
#endif

static int is_pseudo_fs(const char *fstype)
{
    static const char *blocked[] = {
        "proc", "sysfs", "cgroup", "cgroup2", "devtmpfs", "devpts", "mqueue",
        "debugfs", "tracefs", "securityfs", "pstore", "bpf", "configfs",
        "fusectl", "hugetlbfs", "efivarfs", "autofs", "binfmt_misc",
        "rpc_pipefs", "ramfs", "nsfs", "selinuxfs", "sunrpc", "tmpfs",
        "fuse.gvfsd-fuse", "fuse.portal", "overlay", "squashfs", NULL
    };
    int i;

    for (i = 0; blocked[i] != NULL; i++) {
        if (strcmp(fstype, blocked[i]) == 0) {
            return 1;
        }
    }
    return 0;
}

/* Decodes the octal escapes (\040 etc.) that /proc/mounts uses for spaces. */
static void unescape_mount(const char *in, char *out, size_t out_size)
{
    size_t o = 0;

    while (*in != '\0' && o + 1 < out_size) {
        if (in[0] == '\\' && in[1] >= '0' && in[1] <= '7' &&
            in[2] >= '0' && in[2] <= '7' && in[3] >= '0' && in[3] <= '7') {
            int value = (in[1] - '0') * 64 + (in[2] - '0') * 8 + (in[3] - '0');
            out[o++] = (char)value;
            in += 4;
        } else {
            out[o++] = *in++;
        }
    }
    out[o] = '\0';
}

int cow_disk_read_mounts(const char *proc_root, CowMount *mounts, int max, int *count)
{
    char path[COWTOP_PATH_MAX];
    char line[2048];
    FILE *file;
    int n = 0;

    *count = 0;

    if (snprintf(path, sizeof(path), "%s/mounts", proc_root) >= (int)sizeof(path)) {
        return -1;
    }

    file = fopen(path, "r");
    if (file == NULL) {
        return -1;
    }

    while (fgets(line, sizeof(line), file) != NULL && n < max) {
        char source[256];
        char raw_mount[512];
        char mount[COW_PATH];
        char fstype[COW_NAME];
        struct statvfs vfs;
        unsigned long long total_kb;
        unsigned long long avail_kb;
        unsigned long long used_kb;

        if (sscanf(line, "%255s %511s %63s", source, raw_mount, fstype) != 3) {
            continue;
        }
        if (is_pseudo_fs(fstype)) {
            continue;
        }

        unescape_mount(raw_mount, mount, sizeof(mount));

        if (statvfs(mount, &vfs) != 0 || vfs.f_blocks == 0) {
            continue;
        }

        total_kb = (unsigned long long)vfs.f_blocks * vfs.f_frsize / 1024ULL;
        avail_kb = (unsigned long long)vfs.f_bavail * vfs.f_frsize / 1024ULL;
        used_kb = (unsigned long long)(vfs.f_blocks - vfs.f_bfree) * vfs.f_frsize / 1024ULL;
        if (total_kb == 0) {
            continue;
        }

        snprintf(mounts[n].source, sizeof(mounts[n].source), "%s", source);
        snprintf(mounts[n].mount, sizeof(mounts[n].mount), "%s", mount);
        snprintf(mounts[n].fstype, sizeof(mounts[n].fstype), "%s", fstype);
        mounts[n].total_kb = total_kb;
        mounts[n].used_kb = used_kb;
        mounts[n].avail_kb = avail_kb;
        mounts[n].used_percent = (used_kb + avail_kb) > 0
            ? 100.0 * (double)used_kb / (double)(used_kb + avail_kb)
            : 0.0;
        n++;
    }

    fclose(file);
    *count = n;
    return 0;
}

static int is_partition(const char *name)
{
    size_t len = strlen(name);
    size_t i;
    size_t letters = 0;

    /* nvme0n1p3 / mmcblk0p1 style: a 'p' preceded by a digit, then digits. */
    for (i = 1; i + 1 < len; i++) {
        if (name[i] == 'p' && isdigit((unsigned char)name[i - 1])) {
            size_t j;
            int all_digits = 1;
            for (j = i + 1; j < len; j++) {
                if (!isdigit((unsigned char)name[j])) {
                    all_digits = 0;
                    break;
                }
            }
            if (all_digits) {
                return 1;
            }
        }
    }

    /* sda1 / vdb2 style: leading letters followed only by digits. */
    while (letters < len && isalpha((unsigned char)name[letters])) {
        letters++;
    }
    if (letters > 0 && letters < len) {
        for (i = letters; i < len; i++) {
            if (!isdigit((unsigned char)name[i])) {
                return 0;
            }
        }
        return 1;
    }

    return 0;
}

static int is_virtual_device(const char *name)
{
    static const char *prefixes[] = { "loop", "ram", "dm-", "sr", "zram", "md", "fd", NULL };
    int i;

    for (i = 0; prefixes[i] != NULL; i++) {
        if (strncmp(name, prefixes[i], strlen(prefixes[i])) == 0) {
            return 1;
        }
    }
    return 0;
}

int cow_disk_read_io(const char *proc_root,
                     unsigned long long *read_sectors,
                     unsigned long long *write_sectors)
{
    char path[COWTOP_PATH_MAX];
    char line[1024];
    FILE *file;

    *read_sectors = 0;
    *write_sectors = 0;

    if (snprintf(path, sizeof(path), "%s/diskstats", proc_root) >= (int)sizeof(path)) {
        return -1;
    }

    file = fopen(path, "r");
    if (file == NULL) {
        return -1;
    }

    while (fgets(line, sizeof(line), file) != NULL) {
        unsigned int major, minor;
        char name[64];
        unsigned long long rd_ios, rd_merged, rd_sectors;
        unsigned long long rd_ms, wr_ios, wr_merged, wr_sectors;

        int parsed = sscanf(line,
                            " %u %u %63s %llu %llu %llu %llu %llu %llu %llu",
                            &major, &minor, name, &rd_ios, &rd_merged,
                            &rd_sectors, &rd_ms, &wr_ios, &wr_merged, &wr_sectors);
        if (parsed < 10) {
            continue;
        }
        if (is_virtual_device(name) || is_partition(name)) {
            continue;
        }

        *read_sectors += rd_sectors;
        *write_sectors += wr_sectors;
    }

    fclose(file);
    return 0;
}
