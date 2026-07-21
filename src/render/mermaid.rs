//! Mermaid code blocks: a "view in browser" deep link on the block label.
//!
//! Rendering mermaid inside the terminal was investigated and rejected:
//! mermaid.js cannot lay out without a browser engine (its layout measures
//! rendered text via getBBox), and a faithful text-grid layout engine is a
//! multi-thousand-line subsystem — both at odds with a lean single binary.
//! Instead the viewer makes the fence's label line clickable: it opens the
//! diagram rendered by the official mermaid.live viewer. The entire diagram
//! source travels inside the URL *fragment*, which browsers never send to
//! the server — the code leaves the machine only into the local browser.
//!
//! Fragment format (mermaid-live-editor `serde.ts`): `base64:` +
//! base64url-without-padding of the JSON editor state. The state's
//! `mermaid` field is a JSON string (config-as-text), not a nested object.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

/// Windows `cmd /C start` rejects command lines beyond ~8k characters;
/// diagrams that would exceed it simply don't get the affordance.
const MAX_URL_LEN: usize = 8000;

/// Deep link to the mermaid.live read-only viewer for `code`.
pub fn live_view_url(code: &str) -> Option<String> {
    let state = format!(
        r#"{{"code":{},"mermaid":"{{\"theme\":\"default\"}}","updateDiagram":true,"rough":false}}"#,
        json_string(code)
    );
    let url = format!(
        "https://mermaid.live/view#base64:{}",
        URL_SAFE_NO_PAD.encode(state.as_bytes())
    );
    (url.len() <= MAX_URL_LEN).then_some(url)
}

/// Minimal JSON string encoder (RFC 8259): quote, backslash and control
/// characters escaped; everything else passes through as UTF-8.
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::parser::parse;
    use crate::render::inline::LinkMode;
    use crate::render::theme::{CharSet, Theme};
    use crate::render::{RenderedLine, Renderer};

    fn render(src: &str, width: usize, interactive: bool) -> Vec<RenderedLine> {
        let renderer = Renderer {
            theme: Theme::dark(CharSet::unicode()),
            highlighter: crate::render::highlight::Highlighter::new(false, false),
            link_mode: LinkMode::TextOnly,
            prose_cap: 0,
            margin: 0,
            interactive,
        };
        renderer.render(&parse(src).blocks, width)
    }

    const FENCE: &str = "```mermaid\nflowchart LR\n  A --> B\n```\n";

    #[test]
    fn viewer_label_line_carries_the_live_link() {
        let lines = render(FENCE, 60, true);
        let label = &lines[0];
        assert!(label.plain_text().contains("mermaid"), "label line first");
        assert!(
            label.plain_text().contains("view in browser"),
            "{:?}",
            label.plain_text()
        );
        assert_eq!(label.links.len(), 1);
        let link = &label.links[0];
        assert!(
            link.target.starts_with("https://mermaid.live/view#base64:"),
            "{}",
            link.target
        );
        // The clickable range is exactly the affordance text.
        assert_eq!(&label.plain_text()[link.start..link.end], "view in browser");
        // The diagram body itself carries no links.
        assert!(lines[1..].iter().all(|rl| rl.links.is_empty()));
    }

    #[test]
    fn print_mode_output_is_unchanged() {
        let interactive = render(FENCE, 60, true);
        let print = render(FENCE, 60, false);
        assert!(print.iter().all(|rl| rl.links.is_empty()));
        assert!(!print[0].plain_text().contains("view in browser"));
        // Only the label line differs between the two frontends.
        for (a, b) in interactive.iter().zip(&print).skip(1) {
            assert_eq!(a.plain_text(), b.plain_text());
        }
    }

    #[test]
    fn narrow_widths_drop_the_affordance_not_the_block() {
        let lines = render(FENCE, 22, true);
        assert!(lines[0].plain_text().contains("mermaid"));
        assert!(!lines[0].plain_text().contains("view in browser"));
        assert!(lines[0].links.is_empty());
    }

    #[test]
    fn other_languages_are_untouched() {
        let lines = render("```rust\nfn main() {}\n```\n", 60, true);
        assert!(lines.iter().all(|rl| rl.links.is_empty()));
    }

    #[test]
    fn url_encodes_editor_state_exactly() {
        let url = live_view_url("flowchart LR\n  A --> \"B\"").expect("small diagram gets a url");
        let fragment = url
            .strip_prefix("https://mermaid.live/view#base64:")
            .expect("viewer route + base64 serde prefix");
        let state = URL_SAFE_NO_PAD.decode(fragment).expect("valid base64url");
        assert_eq!(
            String::from_utf8(state).unwrap(),
            r#"{"code":"flowchart LR\n  A --> \"B\"","mermaid":"{\"theme\":\"default\"}","updateDiagram":true,"rough":false}"#
        );
    }

    #[test]
    fn json_escapes_are_complete() {
        assert_eq!(
            json_string("a\"b\\c\nd\re\tf\u{1}"),
            r#""a\"b\\c\nd\re\tf\u0001""#
        );
        // Non-ASCII passes through unescaped (JSON is UTF-8).
        assert_eq!(json_string("日本語 🚀"), "\"日本語 🚀\"");
    }

    #[test]
    fn oversized_diagrams_get_no_url() {
        let huge = "A --> B\n".repeat(2000);
        assert_eq!(live_view_url(&huge), None);
    }
}
