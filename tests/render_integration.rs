//! Integration tests: run the real binary in print mode over fixtures and
//! verify global invariants (no overflow, no panics, content preserved).
//!
//! These tests intentionally assert *properties*, not exact bytes: the point
//! is that any future theme/layout change keeps the output structurally sound.

use std::process::Command;

fn render(fixture: &str, extra: &[&str]) -> String {
    let bin = env!("CARGO_BIN_EXE_mdpad");
    let out = Command::new(bin)
        .arg("--print")
        .arg("--color")
        .arg("never")
        .args(extra)
        .arg(format!(
            "{}/tests/fixtures/{fixture}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .output()
        .expect("binary runs");
    assert!(
        out.status.success(),
        "mdpad failed on {fixture}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 output")
}

fn max_display_width(s: &str) -> usize {
    s.lines()
        .map(unicode_width::UnicodeWidthStr::width)
        .max()
        .unwrap_or(0)
}

#[test]
fn showcase_fits_all_widths() {
    for width in [30, 44, 60, 80, 100, 132, 200] {
        let out = render("showcase.md", &["--width", &width.to_string()]);
        let max = max_display_width(&out);
        assert!(
            max <= width,
            "width {width}: line overflows to {max} cols:\n{}",
            out.lines()
                .filter(|l| unicode_width::UnicodeWidthStr::width(*l) > width)
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

#[test]
fn report_fits_all_widths() {
    for width in [40, 72, 80, 100, 140] {
        let out = render("report.md", &["--width", &width.to_string()]);
        let max = max_display_width(&out);
        assert!(max <= width, "width {width}: overflow to {max}");
    }
}

#[test]
fn report_table_content_survives() {
    // Every model name and numeric value must appear in the output. Cells
    // wrap across bordered lines, so compare alphanumeric streams (borders,
    // spaces and newlines stripped from both sides).
    let alnum = |s: &str| -> String { s.chars().filter(|c| c.is_alphanumeric()).collect() };
    let out = alnum(&render("report.md", &["--width", "80"]));
    for needle in [
        "LiquidAI/LFM2.5-350M:latest",
        "openbmb/minicpm5:latest",
        "215.08",
        "skipped_model_broken",
        "4689.49",
        "model requires more system memory",
    ] {
        assert!(
            out.contains(&alnum(needle)),
            "missing table content {needle:?} at width 80"
        );
    }
}

#[test]
fn showcase_inline_content_survives() {
    let out = render("showcase.md", &["--width", "100"]);
    for needle in [
        "bold italic",
        "strikethrough",
        "named link",
        "an open task",
        "a finished task",
        "Quoted inside a list item.",
        "fn fib(n: u64) -> u64",
        "终端里的漂亮排版是这个工具存在的意义。",
        "emoji cell",
        "raw html renders dimmed, verbatim",
    ] {
        assert!(out.contains(needle), "missing {needle:?} in output");
    }
}

#[test]
fn ansi_output_is_terminated_per_line() {
    let bin = env!("CARGO_BIN_EXE_mdpad");
    let out = Command::new(bin)
        .args(["--print", "--color", "always", "--width", "90"])
        .arg(format!(
            "{}/tests/fixtures/showcase.md",
            env!("CARGO_MANIFEST_DIR")
        ))
        .output()
        .expect("binary runs");
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("\x1b["), "expected ANSI escapes");
    // No line may leak an open style into the next (each line self-contained).
    for line in text.lines().filter(|l| l.contains("\x1b[")) {
        assert!(
            line.rfind("\x1b[0m").is_some(),
            "line lacks reset: {line:?}"
        );
    }
}

#[test]
fn ordered_task_lists_keep_their_numbers() {
    use std::io::Write;
    let bin = env!("CARGO_BIN_EXE_mdpad");
    let mut child = Command::new(bin)
        .args(["--print", "--color", "never", "--width", "60", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"1. [ ] write tests\n2. [x] ship it\n3. profit\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("1. ☐ write tests"), "{text}");
    assert!(text.contains("2. ✔ ship it"), "{text}");
    assert!(text.contains("3."), "{text}");
}

#[test]
fn stdin_pipe_works() {
    use std::io::Write;
    let bin = env!("CARGO_BIN_EXE_mdpad");
    let mut child = Command::new(bin)
        .args(["--print", "--width", "60", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"# Hi\n\n| a | b |\n|---|---|\n| 1 | 2 |\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("Hi"));
    assert!(text.contains('│') || text.contains('|'), "table rendered");
}

#[test]
fn empty_and_pathological_inputs_do_not_crash() {
    use std::io::Write;
    let bin = env!("CARGO_BIN_EXE_mdpad");
    let deep_quotes_cjk = format!("{}你好", "> ".repeat(15)); // regression: infinite wrap loop
    let extreme_nesting = format!("{}deep", "> ".repeat(20_000)); // regression: stack overflow
    let cases: &[&str] = &[
        "",
        "\n\n\n",
        "| | | |\n|---|---|---|\n| | | |",
        "```\nunclosed fence",
        "###### deep heading only",
        "> > > > > > deep quote",
        "- - - - -",
        "**unclosed emphasis [broken link](",
        &"x".repeat(10_000),
        &"- item\n".repeat(2_000),
        &deep_quotes_cjk,
        &extreme_nesting,
    ];
    for (i, case) in cases.iter().enumerate() {
        let mut child = Command::new(bin)
            .args(["--print", "--width", "40", "-"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .spawn()
            .expect("spawn");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(case.as_bytes())
            .unwrap();
        let status = child.wait().unwrap();
        assert!(status.success(), "case {i} crashed");
    }
}
