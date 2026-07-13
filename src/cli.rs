//! Command-line interface definition.

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "mdpad",
    version,
    about = "A fast, beautiful markdown reader and editor for the terminal",
    long_about = None
)]
pub struct Args {
    /// Markdown file to open ("-" reads from stdin)
    pub file: Option<PathBuf>,

    /// Print rendered markdown to stdout instead of opening the viewer
    #[arg(short, long)]
    pub print: bool,

    /// Render width in columns (default: terminal width; minimum 20)
    #[arg(short, long)]
    pub width: Option<usize>,

    /// Color output: auto strips colors when stdout is piped
    #[arg(long, value_enum, default_value_t = ColorChoice::Auto)]
    pub color: ColorChoice,

    /// Use the light theme (default: dark)
    #[arg(long)]
    pub light: bool,

    /// ASCII-only glyphs (no box drawing / unicode bullets)
    #[arg(long)]
    pub ascii: bool,

    /// Show link URLs inline after the link text
    #[arg(long)]
    pub urls: bool,

    /// Cap prose line length for readability (0 = use full width)
    #[arg(long, default_value_t = 100)]
    pub prose_width: usize,

    /// Disable syntax highlighting in code blocks
    #[arg(long)]
    pub no_highlight: bool,

    /// Disable mouse capture in the viewer (keeps native text selection)
    #[arg(long)]
    pub no_mouse: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

impl Args {
    /// Resolve color enablement for print mode.
    pub fn colors_enabled(&self, stdout_is_tty: bool) -> bool {
        // NO_COLOR (https://no-color.org) wins over auto-detection.
        let no_color_env = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
        match self.color {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => stdout_is_tty && !no_color_env,
        }
    }
}
