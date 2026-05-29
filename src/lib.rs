//! cowtop: a cow-themed terminal system monitor.
//!
//! The modules are exposed as a library so integration tests can render the
//! TUI against live `/proc` data without a TTY; `main.rs` is a thin binary on
//! top of the same modules.

pub mod app;
pub mod cow;
pub mod ffi;
pub mod sys;
pub mod ui;
