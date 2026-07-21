mod app;
mod cli;
mod markdown;
mod render;
mod ui;

use std::io::{IsTerminal, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use cli::Args;
use render::Renderer;
use render::highlight::Highlighter;
use render::inline::LinkMode;
use render::theme::{CharSet, Theme};

fn main() -> ExitCode {
    let args = Args::parse();
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("mdpad: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let stdin_tty = std::io::stdin().is_terminal();
    let stdout_tty = std::io::stdout().is_terminal();

    // Resolve the input source.
    let (source, path): (String, Option<PathBuf>) = match &args.file {
        Some(p) if p.as_os_str() == "-" => (read_stdin()?, None),
        Some(p) => (
            std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?,
            Some(p.clone()),
        ),
        None if !stdin_tty => (read_stdin()?, None),
        None => {
            return Err("no input: pass a markdown file (or pipe content in)".into());
        }
    };

    // Print mode when asked for, or when stdout is not a terminal (pipe).
    if args.print || !stdout_tty {
        let colors = args.colors_enabled(stdout_tty);
        let renderer = build_renderer(&args, colors, /* tui: */ false);
        let width = args.width.unwrap_or_else(|| detect_width(stdout_tty));
        let blocks = markdown::parser::parse(&source).blocks;
        let lines = renderer.render(&blocks, width);
        let out = render::ansi::to_ansi(&lines, colors);
        // Broken pipes (e.g. `mdpad x.md | head`) are a normal exit.
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();
        let _ = stdout.write_all(out.as_bytes());
        let _ = stdout.flush();
        return Ok(());
    }

    // Interactive viewer needs a real tty on stdin to read keys. When the
    // document came from a pipe, reattach stdin to the terminal (unix).
    if !stdin_tty && !reattach_tty() {
        // Can't get a tty (CI, some Windows shells): degrade to print mode.
        let colors = args.colors_enabled(stdout_tty);
        let renderer = build_renderer(&args, colors, false);
        let width = args.width.unwrap_or_else(|| detect_width(stdout_tty));
        let blocks = markdown::parser::parse(&source).blocks;
        let lines = renderer.render(&blocks, width);
        print!("{}", render::ansi::to_ansi(&lines, colors));
        return Ok(());
    }

    // NO_COLOR / --color never apply to the viewer too, not just print mode.
    let renderer = build_renderer(&args, args.colors_enabled(true), true);
    app::run(source, path, renderer, &args)?;
    Ok(())
}

fn read_stdin() -> std::io::Result<String> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

fn build_renderer(args: &Args, colors: bool, tui: bool) -> Renderer {
    let chars = if args.ascii {
        CharSet::ascii()
    } else {
        CharSet::unicode()
    };
    let theme = if !colors {
        Theme::plain(chars)
    } else if args.light {
        Theme::light(chars)
    } else {
        Theme::dark(chars)
    };
    let truecolor = std::env::var("COLORTERM")
        .map(|v| v.contains("truecolor") || v.contains("24bit"))
        .unwrap_or(false);
    // In print mode, hiding URLs would make links unusable text; show them
    // unless the user opted out by not passing --urls in the TUI (where a
    // toggle exists instead).
    let link_mode = if args.urls || !tui {
        LinkMode::WithUrl
    } else {
        LinkMode::TextOnly
    };
    Renderer {
        theme,
        highlighter: Highlighter::new(!args.no_highlight && colors, truecolor),
        link_mode,
        prose_cap: args.prose_width,
        margin: 2,
        interactive: tui,
    }
}

fn detect_width(stdout_tty: bool) -> usize {
    if stdout_tty && let Ok((w, _)) = ratatui::crossterm::terminal::size() {
        return w as usize;
    }
    // Piped: honor COLUMNS if the shell exports it, else a sane default.
    std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&w| w >= 20)
        .unwrap_or(100)
}

/// Reattach stdin to the controlling terminal after the document was piped in.
#[cfg(unix)]
fn reattach_tty() -> bool {
    use std::os::fd::AsRawFd;
    let Ok(tty) = std::fs::File::open("/dev/tty") else {
        return false;
    };
    // SAFETY: dup2 on a freshly opened, valid fd onto stdin.
    unsafe { libc::dup2(tty.as_raw_fd(), 0) != -1 }
}

#[cfg(not(unix))]
fn reattach_tty() -> bool {
    // On Windows, crossterm reads events from the console handle directly,
    // which works even when stdin was redirected.
    true
}
