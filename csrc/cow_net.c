#define _POSIX_C_SOURCE 200809L

#include "cow_net.h"

#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#ifndef COWTOP_PATH_MAX
#define COWTOP_PATH_MAX 4096
#endif

int cow_net_read_ifaces(const char *proc_root, CowIface *ifaces, int max, int *count)
{
    char path[COWTOP_PATH_MAX];
    char line[1024];
    FILE *file;
    int n = 0;

    *count = 0;

    if (snprintf(path, sizeof(path), "%s/net/dev", proc_root) >= (int)sizeof(path)) {
        return -1;
    }

    file = fopen(path, "r");
    if (file == NULL) {
        return -1;
    }

    /* Skip the two header lines. */
    if (fgets(line, sizeof(line), file) == NULL ||
        fgets(line, sizeof(line), file) == NULL) {
        fclose(file);
        return -1;
    }

    while (fgets(line, sizeof(line), file) != NULL && n < max) {
        char *colon = strchr(line, ':');
        char *name_start = line;
        unsigned long long nums[16] = {0};
        int parsed;
        size_t name_len;

        if (colon == NULL) {
            continue;
        }

        while (*name_start == ' ' || *name_start == '\t') {
            name_start++;
        }

        name_len = (size_t)(colon - name_start);
        if (name_len == 0 || name_len >= COW_NAME) {
            continue;
        }

        parsed = sscanf(colon + 1,
                        "%llu %llu %llu %llu %llu %llu %llu %llu"
                        " %llu %llu %llu %llu %llu %llu %llu %llu",
                        &nums[0], &nums[1], &nums[2], &nums[3],
                        &nums[4], &nums[5], &nums[6], &nums[7],
                        &nums[8], &nums[9], &nums[10], &nums[11],
                        &nums[12], &nums[13], &nums[14], &nums[15]);
        if (parsed < 9) {
            continue;
        }

        memcpy(ifaces[n].name, name_start, name_len);
        ifaces[n].name[name_len] = '\0';
        ifaces[n].rx_bytes = nums[0];
        ifaces[n].tx_bytes = nums[8];
        ifaces[n].rx_bps = 0.0;
        ifaces[n].tx_bps = 0.0;
        n++;
    }

    fclose(file);
    *count = n;
    return 0;
}

static const char *tcp_state_name(unsigned int st)
{
    switch (st) {
    case 0x01: return "ESTAB";
    case 0x02: return "SYN_SENT";
    case 0x03: return "SYN_RECV";
    case 0x04: return "FIN_WAIT1";
    case 0x05: return "FIN_WAIT2";
    case 0x06: return "TIME_WAIT";
    case 0x07: return "CLOSE";
    case 0x08: return "CLOSE_WAIT";
    case 0x09: return "LAST_ACK";
    case 0x0A: return "LISTEN";
    case 0x0B: return "CLOSING";
    default:   return "UNKNOWN";
    }
}

static void format_ipv4(const char *hex, unsigned int port, char *out, size_t out_size)
{
    unsigned int v = (unsigned int)strtoul(hex, NULL, 16);
    snprintf(out, out_size, "%u.%u.%u.%u:%u",
             v & 0xFFu, (v >> 8) & 0xFFu, (v >> 16) & 0xFFu, (v >> 24) & 0xFFu, port);
}

static void format_ipv6(const char *hex, unsigned int port, char *out, size_t out_size)
{
    unsigned char bytes[16] = {0};
    int i;
    char addr[40];
    int pos = 0;

    if (strlen(hex) < 32) {
        snprintf(out, out_size, "[::]:%u", port);
        return;
    }

    for (i = 0; i < 16; i++) {
        char pair[3] = { hex[i * 2], hex[i * 2 + 1], '\0' };
        bytes[i] = (unsigned char)strtoul(pair, NULL, 16);
    }

    /* Each of the four 32-bit words is little-endian on the wire. */
    for (i = 0; i < 4; i++) {
        unsigned char *w = &bytes[i * 4];
        unsigned char t;
        t = w[0]; w[0] = w[3]; w[3] = t;
        t = w[1]; w[1] = w[2]; w[2] = t;
    }

    for (i = 0; i < 8; i++) {
        unsigned int group = ((unsigned int)bytes[i * 2] << 8) | bytes[i * 2 + 1];
        pos += snprintf(addr + pos, sizeof(addr) - (size_t)pos, "%s%x", i == 0 ? "" : ":", group);
        if (pos >= (int)sizeof(addr) - 1) {
            break;
        }
    }

    snprintf(out, out_size, "[%s]:%u", addr, port);
}

static void read_conn_file(const char *proc_root,
                           const char *leaf,
                           const char *proto,
                           int is_v6,
                           int is_tcp,
                           CowNet *summary,
                           CowConn *conns,
                           int max,
                           int *count)
{
    char path[COWTOP_PATH_MAX];
    char line[2048];
    FILE *file;

    if (snprintf(path, sizeof(path), "%s/net/%s", proc_root, leaf) >= (int)sizeof(path)) {
        return;
    }

    file = fopen(path, "r");
    if (file == NULL) {
        return;
    }

    /* Header line. */
    if (fgets(line, sizeof(line), file) == NULL) {
        fclose(file);
        return;
    }

    while (fgets(line, sizeof(line), file) != NULL) {
        char local_hex[40];
        char rem_hex[40];
        unsigned int local_port = 0;
        unsigned int rem_port = 0;
        unsigned int state = 0;
        int uid = 0;
        unsigned long inode = 0;
        int sl;

        int parsed = sscanf(line,
                            " %d: %39[0-9A-Fa-f]:%x %39[0-9A-Fa-f]:%x %x"
                            " %*x:%*x %*x:%*x %*x %d %*d %lu",
                            &sl, local_hex, &local_port, rem_hex, &rem_port,
                            &state, &uid, &inode);
        if (parsed < 6) {
            continue;
        }

        if (is_tcp) {
            switch (state) {
            case 0x01: summary->tcp_estab++; break;
            case 0x0A: summary->tcp_listen++; break;
            case 0x06: summary->tcp_time_wait++; break;
            default:   summary->tcp_other++; break;
            }
        } else {
            summary->udp_count++;
        }

        if (*count >= max) {
            continue;
        }

        {
            CowConn *c = &conns[*count];
            snprintf(c->proto, sizeof(c->proto), "%s", proto);
            if (is_v6) {
                format_ipv6(local_hex, local_port, c->local, sizeof(c->local));
                format_ipv6(rem_hex, rem_port, c->remote, sizeof(c->remote));
            } else {
                format_ipv4(local_hex, local_port, c->local, sizeof(c->local));
                format_ipv4(rem_hex, rem_port, c->remote, sizeof(c->remote));
            }
            if (is_tcp) {
                snprintf(c->state, sizeof(c->state), "%s", tcp_state_name(state));
            } else {
                snprintf(c->state, sizeof(c->state), "%s", state == 0x07 ? "LISTEN" : "ACTIVE");
            }
            c->uid = uid;
            c->inode = inode;
            (*count)++;
        }
    }

    fclose(file);
}

int cow_net_read_conns(const char *proc_root,
                       CowNet *summary,
                       CowConn *conns,
                       int max,
                       int *count)
{
    summary->tcp_estab = 0;
    summary->tcp_listen = 0;
    summary->tcp_time_wait = 0;
    summary->tcp_other = 0;
    summary->udp_count = 0;
    *count = 0;

    read_conn_file(proc_root, "tcp", "tcp", 0, 1, summary, conns, max, count);
    read_conn_file(proc_root, "tcp6", "tcp6", 1, 1, summary, conns, max, count);
    read_conn_file(proc_root, "udp", "udp", 0, 0, summary, conns, max, count);
    read_conn_file(proc_root, "udp6", "udp6", 1, 0, summary, conns, max, count);

    return 0;
}
