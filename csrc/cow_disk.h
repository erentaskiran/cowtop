#ifndef COWTOP_COW_DISK_H
#define COWTOP_COW_DISK_H

#include "cowsys.h"

/* Reads real (non-pseudo) filesystems from <proc_root>/mounts and queries
 * capacity via statvfs on the live mount point. Returns 0 on success. */
int cow_disk_read_mounts(const char *proc_root,
                         CowMount *mounts,
                         int max,
                         int *count);

/* Sums sectors read/written across whole-disk block devices in
 * <proc_root>/diskstats. Caller converts to a rate using two samples. */
int cow_disk_read_io(const char *proc_root,
                     unsigned long long *read_sectors,
                     unsigned long long *write_sectors);

#endif
