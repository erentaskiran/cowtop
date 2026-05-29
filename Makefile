# cowtop — Rust/ratatui TUI system monitor with a C /proc backend.

CARGO ?= cargo

.PHONY: all build release run test clean

all: build

build:
	$(CARGO) build

release:
	$(CARGO) build --release

run:
	$(CARGO) run -- $(ARGS)

test:
	$(CARGO) test

clean:
	$(CARGO) clean
	rm -f report.txt
