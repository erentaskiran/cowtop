use std::time::{Duration, Instant};

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use cowtui::app::App;
use cowtui::sys::Monitor;
use cowtui::{ffi, ui};

struct Args {
    proc_root: Option<String>,
    interval: Duration,
    top: usize,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        proc_root: None,
        interval: Duration::from_millis(1000),
        top: ffi::COW_MAX_PROCS,
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            "-i" | "--interval" => {
                let v = it.next().ok_or("-i requires seconds")?;
                let secs: f64 = v.parse().map_err(|_| format!("invalid interval: {v}"))?;
                if secs <= 0.0 || secs > 3600.0 {
                    return Err(format!("interval out of range: {v}"));
                }
                args.interval = Duration::from_millis((secs * 1000.0) as u64);
            }
            "-n" | "--top" => {
                let v = it.next().ok_or("-n requires a count")?;
                let n: usize = v.parse().map_err(|_| format!("invalid count: {v}"))?;
                args.top = n.clamp(1, ffi::COW_MAX_PROCS);
            }
            "--proc-root" => {
                args.proc_root = Some(it.next().ok_or("--proc-root requires a path")?);
            }
            other => return Err(format!("unknown option: {other}")),
        }
    }
    Ok(args)
}

fn print_usage() {
    println!(
        "cowtui — a cow-themed terminal system monitor\n\n\
         Usage: cowtui [options]\n\n\
         Options:\n\
         \x20 -i, --interval SECONDS  refresh interval (default 1)\n\
         \x20 -n, --top COUNT         processes per table (default {max})\n\
         \x20     --proc-root PATH    read from another proc root (testing)\n\
         \x20 -h, --help              show this help\n\n\
         Keys: q quit · Tab/←→ switch tabs · 1-5 jump · ↑↓ scroll · p pause",
        max = ffi::COW_MAX_PROCS
    );
}

fn main() -> std::io::Result<()> {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("cowtui: {e}");
            std::process::exit(2);
        }
    };

    let monitor = match Monitor::new(args.proc_root.as_deref()) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("cowtui: {e}");
            std::process::exit(1);
        }
    };

    let mut app = App::new(monitor, args.top);
    // Prime two samples so the first painted frame already has real rates
    // (CPU%, net and disk throughput are all inter-sample deltas).
    app.refresh();
    std::thread::sleep(Duration::from_millis(250));
    app.refresh();

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app, args.interval);
    ratatui::restore();
    result
}

fn run(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    interval: Duration,
) -> std::io::Result<()> {
    let mut last = Instant::now();
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        let timeout = interval.saturating_sub(last.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if ctrl => break,
                    KeyCode::Tab | KeyCode::Right => app.next_tab(),
                    KeyCode::BackTab | KeyCode::Left => app.prev_tab(),
                    KeyCode::Char(c @ '1'..='5') => {
                        app.select_tab(c as usize - '1' as usize);
                    }
                    KeyCode::Char('p') => app.toggle_pause(),
                    KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                    KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                    KeyCode::Char('r') => app.refresh(),
                    _ => {}
                }
            }
        }

        if last.elapsed() >= interval {
            app.refresh();
            last = Instant::now();
        }
    }
    Ok(())
}
