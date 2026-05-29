use std::time::{Duration, Instant};

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};

use cowtop::{ffi, ui};
use cowtop::app::App;
use cowtop::sys::{Monitor, Signal};

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
        "cowtop — a cow-themed terminal system monitor\n\n\
         Usage: cowtop [options]\n\n\
         Options:\n\
         \x20 -i, --interval SECONDS  refresh interval (default 1)\n\
         \x20 -n, --top COUNT         processes per table (default {max})\n\
         \x20     --proc-root PATH    read from another proc root (testing)\n\
         \x20 -h, --help              show this help\n\n\
         Keys:\n\
         \x20 q / Esc      quit\n\
         \x20 Tab / ← →    switch tab   1-6 jump to tab\n\
         \x20 t / T         prev/next theme     ? help\n\
         \x20 /             search processes    s cycle sort\n\
         \x20 k             kill selected proc\n\
         \x20 ↑↓ / jk       scroll      p pause      r refresh\n\n\
         Themes: Pasture, Midnight, Sunset, Forest, Arctic, Retro",
        max = ffi::COW_MAX_PROCS
    );
}

fn main() -> std::io::Result<()> {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("cowtop: {e}");
            std::process::exit(2);
        }
    };

    let monitor = match Monitor::new(args.proc_root.as_deref()) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("cowtop: {e}");
            std::process::exit(1);
        }
    };

    let mut app = App::new(monitor, args.top);
    app.refresh();
    std::thread::sleep(Duration::from_millis(250));
    app.refresh();

    let mut terminal = ratatui::init();
    // Enable mouse capture
    ratatui::crossterm::execute!(
        std::io::stderr(),
        ratatui::crossterm::event::EnableMouseCapture
    )?;
    let result = run(&mut terminal, &mut app, args.interval);
    ratatui::crossterm::execute!(
        std::io::stderr(),
        ratatui::crossterm::event::DisableMouseCapture
    )?;
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
            match event::read()? {
                Event::Key(key) => {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    if app.search_query.is_some() {
                        match key.code {
                            KeyCode::Esc => app.exit_search(),
                            KeyCode::Backspace => { app.search_pop(); }
                            KeyCode::Enter => app.exit_search(),
                            KeyCode::Char(c) => { app.search_push(c); }
                            _ => {}
                        }
                        continue;
                    }
                    if app.show_help {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('?') => app.toggle_help(),
                            _ => {}
                        }
                        continue;
                    }
                    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') if ctrl => break,
                        KeyCode::Tab | KeyCode::Right => app.next_tab(),
                        KeyCode::BackTab | KeyCode::Left => app.prev_tab(),
                        KeyCode::Char(c @ '1'..='6') => {
                            app.select_tab(c as usize - '1' as usize);
                        }
                        KeyCode::Char('p') => app.toggle_pause(),
                        KeyCode::Char('r') => app.refresh(),
                        KeyCode::Char('t') => app.next_theme(),
                        KeyCode::Char('T') => app.prev_theme(),
                        KeyCode::Char('?') => app.toggle_help(),
                        KeyCode::Char('/') => app.enter_search(),
                        KeyCode::Char('s') => app.cycle_sort(),
                        KeyCode::Char('k') => {
                            if let Some(pid) = app.selected_pid {
                                let _ = Monitor::kill_process(pid, Signal::Term);
                                app.selected_pid = None;
                                app.refresh();
                            }
                        }
                        KeyCode::Enter => {
                            // Select first visible process as "selected"
                            let procs = app.filtered_procs(1, true);
                            app.selected_pid = procs.first().map(|p| p.pid);
                        }
                        KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                        KeyCode::Up => app.scroll_up(),
                        _ => {}
                    }
                }
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollDown => app.scroll_down(),
                    MouseEventKind::ScrollUp => app.scroll_up(),
                    _ => {}
                },
                _ => {}
            }
        }

        if last.elapsed() >= interval {
            app.refresh();
            last = Instant::now();
        }
    }
    Ok(())
}
