# cowtop

`cowtop` is a small POSIX C system monitor for Linux. It reads CPU,
memory, and process data directly from `/proc`, then prints a compact
terminal snapshot.

## Build

```sh
make
```

The build uses:

```sh
cc -std=c11 -Wall -Wextra -pedantic -pthread -O2
```

## Usage

```sh
./cowtop
./cowtop -w
./cowtop -w -i 1 -n 10
./cowtop -o report.txt
./cowtop --proc-root tests/fixtures/proc
```

Options:

- `-w`: refresh continuously.
- `-i SECONDS`: refresh interval in seconds, default `2`.
- `-n COUNT`: number of top CPU and memory processes, default `5`.
- `-o PATH`: write the latest snapshot report to a file.
- `--proc-root PATH`: read from another proc root for tests.

On systems without `/proc`, run against the fixture to verify parsing:

```sh
make run ARGS="--proc-root tests/fixtures/proc"
```
