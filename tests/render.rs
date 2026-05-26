//! Renders every tab to an in-memory backend using live /proc data. This
//! catches layout panics, FFI/struct-layout mismatches and obvious garbage
//! without needing a real terminal.

use cowtui::app::{App, Tab};
use cowtui::sys::Monitor;
use cowtui::ui;

use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn buffer_text(terminal: &Terminal<TestBackend>) -> String {
    let buf = terminal.backend().buffer();
    let area = buf.area;
    let mut out = String::new();
    for y in 0..area.height {
        for x in 0..area.width {
            out.push_str(buf[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

fn app_with_data() -> App {
    let monitor = Monitor::new(None).expect("monitor");
    let mut app = App::new(monitor, 32);
    // Two samples so rates and CPU% are populated.
    app.refresh();
    std::thread::sleep(std::time::Duration::from_millis(300));
    app.refresh();
    app
}

#[test]
fn renders_all_tabs_at_several_sizes() {
    let mut app = app_with_data();
    assert!(app.snapshot.cpu.total_percent >= 0.0);
    assert!(app.snapshot.mem.total_kb > 0, "should read real memory");

    for size in [(80u16, 24u16), (120, 40), (200, 60), (60, 20)] {
        for tab in Tab::ALL {
            app.tab = tab;
            let backend = TestBackend::new(size.0, size.1);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|frame| ui::render(frame, &app))
                .expect("draw should not panic");

            let text = buffer_text(&terminal);
            assert!(text.contains("cowtui"), "banner title missing at {size:?}");
        }
    }
}

#[test]
fn overview_shows_panels() {
    let mut app = app_with_data();
    app.tab = Tab::Overview;
    let mut terminal = Terminal::new(TestBackend::new(140, 44)).unwrap();
    terminal.draw(|frame| ui::render(frame, &app)).unwrap();
    let text = buffer_text(&terminal);

    for needle in ["CPU", "Memory pulse", "Network pulse", "Storage", "Packet tracing"] {
        assert!(text.contains(needle), "overview missing panel: {needle}");
    }
}

#[test]
fn dump_overview() {
    // Visible with `cargo test -- --nocapture` for a quick eyeball.
    let mut app = app_with_data();
    app.tab = Tab::Overview;
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    terminal.draw(|frame| ui::render(frame, &app)).unwrap();
    println!("\n{}", buffer_text(&terminal));
}
