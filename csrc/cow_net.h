#ifndef COWTOP_COW_NET_H
#define COWTOP_COW_NET_H

#include "cowsys.h"

/* Reads cumulative rx/tx byte counters for every interface from
 * <proc_root>/net/dev. Fills name/rx_bytes/tx_bytes only; rates are left zero
 * for the caller to derive from two samples. Returns 0 on success. */
int cow_net_read_ifaces(const char *proc_root,
                        CowIface *ifaces,
                        int max,
                        int *count);

/* Reads <proc_root>/net/{tcp,tcp6,udp,udp6}, fills the connection list (up to
 * max) and the aggregate state counters on *summary. Returns 0 on success. */
int cow_net_read_conns(const char *proc_root,
                       CowNet *summary,
                       CowConn *conns,
                       int max,
                       int *count);

#endif
