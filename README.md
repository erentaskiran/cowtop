# cowtui

A cow-themed terminal system monitor for Linux. The number crunching lives in a
small POSIX C backend that reads straight from `/proc`; the interface is a
[ratatui](https://ratatui.rs) TUI in Rust that the C library is linked into via
FFI. The mascot cow grazes when the box is idle and panics when it is not.

```
        ^__^      c o w t u i
        (oo)\___________
        (__)\          )\/\
            ||--------w |
```

## Features

- **Cow banner** — ASCII cow whose face and colour track load (grazing → panic).
- **CPU** — aggregate gauge, history sparkline, and a per-core gauge grid.
- **Memory pulse** — RAM + swap gauges, cache/buffer/available, history sparkline.
- **Network pulse** — per-interface rx/tx rates and live throughput sparklines.
- **Packet tracing** — decoded TCP/UDP (v4 + v6) sockets with state and owner UID.
- **Storage** — per-filesystem usage (via `statvfs`) plus disk read/write IO rate.
- **Processes** — top consumers by CPU and by memory, side by side.

## Build

```sh
make            # or: cargo build --release
```

Requires a C compiler (the `cc` crate compiles `csrc/` at build time) and a
recent Rust toolchain.

## Run

```sh
cargo run --release
./target/release/cowtui
./target/release/cowtui -i 2 -n 30
./target/release/cowtui --proc-root tests/fixtures/proc
```

Options:

- `-i, --interval SECONDS` — refresh interval (default `1`).
- `-n, --top COUNT` — processes per table (default `64`).
- `--proc-root PATH` — read from another proc root, for testing.

Keys: `q` quit · `Tab`/`←`/`→` switch tabs · `1`-`5` jump · `↑`/`↓` scroll ·
`p` pause · `r` force refresh.

## Architecture

```
csrc/            C backend (no Rust knowledge)
  cowsys.[ch]    monitor handle + flat FFI sample; computes inter-sample rates
  proc_reader.*  CPU aggregate, memory, process table (with CPU% deltas)
  cow_net.*      /proc/net/dev counters + /proc/net/{tcp,tcp6,udp,udp6}
  cow_disk.*     /proc/mounts + statvfs, /proc/diskstats IO
  cli_main.c     legacy standalone text snapshot tool (`make cli`)
src/             Rust frontend
  ffi.rs         #[repr(C)] mirror of cowsys.h
  sys.rs         safe wrapper; owns the monitor, converts to owned structs
  app.rs         app state, tabs, sparkline ring buffers
  cow.rs         the mascot: moods and ASCII art
  ui.rs          ratatui rendering
  main.rs        arg parsing + event loop
```

The C side fills one fixed-capacity `CowSample` struct per tick (no heap
ownership crosses the boundary except the opaque monitor handle), so the Rust
mirror is a plain `#[repr(C)]` layout filled by pointer.

## Test

```sh
cargo test
```

Integration tests render every tab to an in-memory backend using live `/proc`
data, catching layout panics and FFI/struct-layout mismatches without a TTY.

## Legacy C tool

The original non-TUI snapshot printer is still here:

```sh
make cli
./cowtop --proc-root tests/fixtures/proc
```
