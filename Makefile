# cowtui — Rust/ratatui frontend over a C /proc backend.
# `make` builds the TUI; `make cli` builds the legacy C snapshot tool.

CARGO ?= cargo
CC ?= cc
CFLAGS ?= -std=c11 -Wall -Wextra -pedantic -pthread -O2
CPPFLAGS ?= -Icsrc
LDFLAGS ?= -pthread

CLI_TARGET := cowtop
CLI_SOURCES := csrc/cli_main.c csrc/proc_reader.c

.PHONY: all build release run test clean cli

all: build

build:
	$(CARGO) build

release:
	$(CARGO) build --release

run:
	$(CARGO) run -- $(ARGS)

test:
	$(CARGO) test

# Legacy non-TUI C snapshot tool (reads the same /proc data, prints text).
cli: $(CLI_TARGET)

$(CLI_TARGET): $(CLI_SOURCES) csrc/proc_reader.h
	$(CC) $(CPPFLAGS) $(CFLAGS) $(CLI_SOURCES) $(LDFLAGS) -o $@

clean:
	$(CARGO) clean
	rm -f $(CLI_TARGET) report.txt
