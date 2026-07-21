//! Table layout: the reason this tool exists.
//!
//! Column sizing follows the browser algorithm in miniature:
//! 1. natural width  = widest unwrapped cell content per column
//! 2. minimum width  = widest single word per column (word wrap floor)
//! 3. fits naturally -> done; else distribute the deficit: start at minimums
//!    and grow toward natural proportionally to remaining need; if even the
//!    minimums overflow, shrink proportionally and hard-break words.
//!
//! Columns with no declared alignment get right-alignment when their body
//! cells are predominantly numeric — the way a human would lay them out.

use ratatui::text::{Line, Span};

use crate::markdown::model::{Alignment, Cell};
use crate::render::inline::{InlineRenderer, LinkMode};
use crate::render::links::LinkRegistry;
use crate::render::theme::Theme;
use crate::render::wrap::{spans_width, wrap_spans};

pub struct TableRenderer<'t> {
    theme: &'t Theme,
    links: &'t LinkRegistry,
}

impl<'t> TableRenderer<'t> {
    pub fn new(theme: &'t Theme, links: &'t LinkRegistry) -> Self {
        Self { theme, links }
    }

    pub fn render(
        &self,
        alignments: &[Alignment],
        head: &[Cell],
        rows: &[Vec<Cell>],
        width: usize,
    ) -> Vec<Line<'static>> {
        // Inside tables, URLs are never appended: they destroy column layout.
        let inline = InlineRenderer::new(self.theme, LinkMode::TextOnly, self.links);

        let ncols = alignments
            .len()
            .max(head.len())
            .max(rows.iter().map(|r| r.len()).max().unwrap_or(0));
        if ncols == 0 {
            return Vec::new();
        }

        // Render all cells to unwrapped spans once.
        let head_spans: Vec<Vec<Span<'static>>> = normalize_row(head, ncols)
            .into_iter()
            .map(|c| inline.render(c, self.theme.table_header))
            .collect();
        let body_spans: Vec<Vec<Vec<Span<'static>>>> = rows
            .iter()
            .map(|r| {
                normalize_row(r, ncols)
                    .into_iter()
                    .map(|c| inline.render(c, self.theme.text))
                    .collect()
            })
            .collect();

        let aligns = effective_alignments(alignments, rows, ncols);
        // No grid stage fits (word minimums overflow even without padding):
        // any grid would be vertical confetti or overflow the width. A
        // record layout (one "Header: value" list per row) reads better.
        let Some((widths, pad)) = column_widths(&head_spans, &body_spans, ncols, width) else {
            return self.render_records(head, rows, width);
        };

        // Wrap every cell to its column width.
        let head_wrapped: Vec<CellLines> = head_spans
            .iter()
            .zip(&widths)
            .map(|(c, w)| wrap_cell(c, *w))
            .collect();
        let body_wrapped: Vec<Vec<CellLines>> = body_spans
            .iter()
            .map(|r| {
                r.iter()
                    .zip(&widths)
                    .map(|(c, w)| wrap_cell(c, *w))
                    .collect()
            })
            .collect();

        let multiline = body_wrapped
            .iter()
            .any(|r: &Vec<CellLines>| r.iter().any(|c| c.len() > 1))
            || head_wrapped.iter().any(|c| c.len() > 1);

        let mut out = Vec::new();
        let has_head = head_spans.iter().any(|c| !c.is_empty());

        out.push(self.border(&widths, pad, BorderKind::Top));
        if has_head {
            self.emit_row(&mut out, &head_wrapped, &widths, &aligns, pad);
            // No separator when there are no body rows: it would read as a
            // phantom empty row.
            if !body_wrapped.is_empty() {
                out.push(self.border(&widths, pad, BorderKind::Mid));
            }
        }
        for (i, row) in body_wrapped.iter().enumerate() {
            self.emit_row(&mut out, row, &widths, &aligns, pad);
            // Separators between rows only when cells wrap: they are then
            // necessary for grouping; otherwise they just waste vertical space.
            if multiline && i + 1 < body_wrapped.len() {
                out.push(self.border(&widths, pad, BorderKind::Mid));
            }
        }
        out.push(self.border(&widths, pad, BorderKind::Bottom));
        out
    }

    fn emit_row(
        &self,
        out: &mut Vec<Line<'static>>,
        cells: &[CellLines],
        widths: &[usize],
        aligns: &[Alignment],
        pad: usize,
    ) {
        let height = cells.iter().map(|c| c.len()).max().unwrap_or(1);
        let bstyle = self.theme.table_border;
        let v = self.theme.chars.v;
        for r in 0..height {
            let mut spans: Vec<Span<'static>> = vec![Span::styled(v.to_string(), bstyle)];
            for (c, cell) in cells.iter().enumerate() {
                let content = cell.get(r).cloned().unwrap_or_default();
                if pad > 0 {
                    spans.push(Span::raw(" ".repeat(pad)));
                }
                spans.extend(align_cell(content, widths[c], aligns[c]));
                if pad > 0 {
                    spans.push(Span::raw(" ".repeat(pad)));
                }
                spans.push(Span::styled(v.to_string(), bstyle));
            }
            out.push(Line::from(spans));
        }
    }

    /// Fallback for tables that cannot physically fit as a grid: each row
    /// becomes a compact record ("Header: value" per line), separated by
    /// short rules. All content survives; comparison across rows is
    /// sacrificed — the only honest option at such widths.
    fn render_records(
        &self,
        head: &[Cell],
        rows: &[Vec<Cell>],
        width: usize,
    ) -> Vec<Line<'static>> {
        let inline = InlineRenderer::new(self.theme, LinkMode::TextOnly, self.links);
        let width = width.max(1);
        let mut out = Vec::new();
        // Header-only table: show the headers themselves, or the table
        // silently vanishes.
        if rows.is_empty() {
            for header in head {
                let spans = inline.render(header, self.theme.table_header);
                for line in wrap_spans(&spans, width) {
                    out.push(Line::from(line));
                }
            }
            return out;
        }
        for (r, row) in rows.iter().enumerate() {
            if r > 0 {
                out.push(Line::from(Span::styled(
                    self.theme.chars.h.repeat(width.min(12)),
                    self.theme.table_border,
                )));
            }
            for (c, cell) in row.iter().enumerate() {
                let mut spans: Vec<Span<'static>> = Vec::new();
                if let Some(header) = head.get(c) {
                    let text = crate::markdown::model::inlines_to_string(header);
                    if !text.trim().is_empty() {
                        spans.push(Span::styled(
                            format!("{}: ", text.trim()),
                            self.theme.table_header,
                        ));
                    }
                }
                spans.extend(inline.render(cell, self.theme.text));
                for line in wrap_spans(&spans, width) {
                    out.push(Line::from(line));
                }
            }
        }
        out
    }

    fn border(&self, widths: &[usize], pad: usize, kind: BorderKind) -> Line<'static> {
        let ch = &self.theme.chars;
        let (l, m, r) = match kind {
            BorderKind::Top => (ch.tl, ch.t_down, ch.tr),
            BorderKind::Mid => (ch.t_right, ch.cross, ch.t_left),
            BorderKind::Bottom => (ch.bl, ch.t_up, ch.br),
        };
        let mut s = String::from(l);
        for (i, w) in widths.iter().enumerate() {
            s.push_str(&ch.h.repeat(w + 2 * pad));
            s.push_str(if i + 1 == widths.len() { r } else { m });
        }
        Line::from(Span::styled(s, self.theme.table_border))
    }
}

enum BorderKind {
    Top,
    Mid,
    Bottom,
}

type CellLines = Vec<Vec<Span<'static>>>;

fn wrap_cell(spans: &[Span<'static>], width: usize) -> CellLines {
    wrap_spans(spans, width.max(1))
}

/// Pad/align one visual cell line to the column width.
fn align_cell(mut spans: Vec<Span<'static>>, width: usize, align: Alignment) -> Vec<Span<'static>> {
    let content = spans_width(&spans);
    let pad = width.saturating_sub(content);
    match align {
        Alignment::Right => {
            let mut out = vec![Span::raw(" ".repeat(pad))];
            out.append(&mut spans);
            out
        }
        Alignment::Center => {
            let left = pad / 2;
            let right = pad - left;
            let mut out = vec![Span::raw(" ".repeat(left))];
            out.append(&mut spans);
            out.push(Span::raw(" ".repeat(right)));
            out
        }
        _ => {
            spans.push(Span::raw(" ".repeat(pad)));
            spans
        }
    }
}

fn normalize_row(row: &[Cell], ncols: usize) -> Vec<&[crate::markdown::model::Inline]> {
    (0..ncols)
        .map(|i| row.get(i).map(|c| c.as_slice()).unwrap_or(&[]))
        .collect()
}

/// Decide effective alignment: declared alignment wins; otherwise columns
/// whose body cells are mostly numeric become right-aligned.
fn effective_alignments(
    declared: &[Alignment],
    rows: &[Vec<Cell>],
    ncols: usize,
) -> Vec<Alignment> {
    (0..ncols)
        .map(
            |c| match declared.get(c).copied().unwrap_or(Alignment::None) {
                Alignment::None => {
                    if column_is_numeric(rows, c) {
                        Alignment::Right
                    } else {
                        Alignment::Left
                    }
                }
                a => a,
            },
        )
        .collect()
}

fn column_is_numeric(rows: &[Vec<Cell>], col: usize) -> bool {
    let mut numeric = 0usize;
    let mut considered = 0usize;
    for row in rows {
        let Some(cell) = row.get(col) else { continue };
        let text = crate::markdown::model::inlines_to_string(cell);
        let text = text.trim();
        if text.is_empty() || is_placeholder(text) {
            continue; // placeholders don't vote
        }
        considered += 1;
        if is_numeric(text) {
            numeric += 1;
        }
    }
    considered > 0 && numeric * 10 >= considered * 7 // >= 70%
}

fn is_placeholder(s: &str) -> bool {
    matches!(s, "—" | "–" | "-" | "n/a" | "N/A" | "?")
}

fn is_numeric(s: &str) -> bool {
    let mut has_digit = false;
    for (i, ch) in s.chars().enumerate() {
        match ch {
            '0'..='9' => has_digit = true,
            '.' | ',' | '%' | '_' => {}
            // Signs only lead a number: internal dashes mean dates
            // (2026-07-13) or phone-like ids, which must stay left-aligned.
            '+' | '-' if i == 0 => {}
            _ => return false,
        }
    }
    has_digit
}

/// Compute column widths for the available content width.
///
/// Overflow policy is staged max-min fairness ("water-filling"): cap only the
/// widest columns at a shared level, leaving narrow columns untouched, with
/// progressively weaker protection floors:
///   A. protect headers and *typical* content (80th percentile): a column
///      holding mostly "ok" doesn't stay 20 wide for one outlier value;
///   B. protect only word-wrap minimums (headers may wrap);
///   C. break words: only the longest tokens get hard-broken.
fn column_widths(
    head: &[Vec<Span<'static>>],
    body: &[Vec<Vec<Span<'static>>>],
    ncols: usize,
    total_width: usize,
) -> Option<(Vec<usize>, usize)> {
    let mut natural = vec![1usize; ncols];
    let mut minimum = vec![1usize; ncols];
    let mut header_w = vec![1usize; ncols];
    let mut body_widths: Vec<Vec<usize>> = vec![Vec::new(); ncols];

    for (c, cell) in head.iter().enumerate() {
        let w = spans_width(cell);
        natural[c] = natural[c].max(w);
        header_w[c] = header_w[c].max(w);
        minimum[c] = minimum[c].max(longest_word(cell));
    }
    for row in body {
        for (c, cell) in row.iter().enumerate() {
            let w = spans_width(cell);
            natural[c] = natural[c].max(w);
            body_widths[c].push(w);
            minimum[c] = minimum[c].max(longest_word(cell));
        }
    }
    // A single giant unbreakable token (URL, model id, hash) must not pin an
    // entire column: word protection stops at 16 columns, beyond that the
    // token hard-breaks like it would in a browser.
    const WORD_PROTECT_CAP: usize = 16;
    for c in 0..ncols {
        minimum[c] = minimum[c].min(natural[c]).min(WORD_PROTECT_CAP);
    }

    // Stage A floors: headers whole + typical body content whole. A column
    // with one outlier value (e.g. one "skipped_model_broken" among "ok"s)
    // is floored at its *typical* width; the outlier wraps.
    let floor_a: Vec<usize> = (0..ncols)
        .map(|c| {
            let typical = percentile_80(&body_widths[c]);
            natural[c].min(typical.max(header_w[c])).max(1)
        })
        .collect();
    // Stage B floors: word-wrap minimums, still capped by typical width so a
    // one-off long token cannot reserve air for a whole column.
    let floor_b: Vec<usize> = (0..ncols)
        .map(|c| floor_a[c].min(minimum[c]).max(1))
        .collect();

    let debug = std::env::var_os("MDPAD_DEBUG_TABLE").is_some();

    // Fixed overhead per stage: ncols+1 border glyphs plus `pad` spaces per
    // cell side. Padding is a luxury worth one downgrade stage: drop it
    // before squeezing content harder. A stage whose overhead alone exceeds
    // the width is skipped honestly — inventing a fake minimum budget here
    // is how tables end up wider than the terminal.
    for (stage, (floors, pad)) in [
        (&floor_a, 1usize),
        (&floor_b, 1),
        (&floor_a, 0),
        (&floor_b, 0),
    ]
    .into_iter()
    .enumerate()
    {
        let fixed = (1 + 2 * pad) * ncols + 1;
        let Some(budget) = total_width.checked_sub(fixed) else {
            continue;
        };
        if natural.iter().sum::<usize>() <= budget {
            return Some((natural, pad));
        }
        if let Some(mut widths) = water_fill(&natural, floors, budget) {
            close_small_gaps(&mut widths, &natural);
            if debug {
                eprintln!(
                    "stage {stage} pad={pad} budget={budget} natural={natural:?} \
                     minimum={minimum:?} floors={floors:?} -> {widths:?}"
                );
            }
            return Some((widths, pad));
        }
    }
    // Word minimums don't fit even unpadded: no readable grid exists.
    None
}

/// Width most body cells fit in (80th percentile, upper-biased).
fn percentile_80(widths: &[usize]) -> usize {
    if widths.is_empty() {
        return 0;
    }
    let mut sorted = widths.to_vec();
    sorted.sort_unstable();
    let idx = ((sorted.len() * 4) / 5).min(sorted.len() - 1);
    sorted[idx]
}

/// Max-min fair fill over the *excess above floors*:
/// width[c] = min(target[c], floor[c] + level). Floors encode deserved need
/// (typical content, headers); the remaining budget accommodates outliers
/// and is shared max-min fairly. None when even the floors overflow.
fn water_fill(target: &[usize], floor: &[usize], budget: usize) -> Option<Vec<usize>> {
    let ncols = target.len();
    let base: usize = (0..ncols).map(|c| target[c].min(floor[c])).sum();
    if base > budget {
        return None;
    }
    let hi = target.iter().copied().max().unwrap_or(1);
    let sum_at =
        |level: usize| -> usize { (0..ncols).map(|c| target[c].min(floor[c] + level)).sum() };
    let level = binary_search_level(0, hi, budget, &sum_at);
    let mut widths: Vec<usize> = (0..ncols)
        .map(|c| target[c].min(floor[c] + level))
        .collect();
    distribute_leftover(&mut widths, target, budget);
    Some(widths)
}

/// A column 2-4 cells short of fitting entirely wraps on EVERY row — the
/// worst outcome per column-width spent. Close such small gaps by shaving a
/// wide column that wraps anyway (one extra wrap row there beats one wrap row
/// per table row here).
fn close_small_gaps(widths: &mut [usize], natural: &[usize]) {
    const MAX_GAP: usize = 2;
    const DONOR_FLOOR: usize = 12;
    let n = widths.len();
    let mut candidates: Vec<usize> = (0..n)
        .filter(|&c| {
            let gap = natural[c].saturating_sub(widths[c]);
            gap > 0 && gap <= MAX_GAP
        })
        .collect();
    candidates.sort_by_key(|&c| natural[c] - widths[c]);
    for c in candidates {
        let gap = natural[c].saturating_sub(widths[c]);
        if gap == 0 {
            continue;
        }
        // Donor: the widest column that already wraps and survives the shave.
        let donor = (0..n)
            .filter(|&d| d != c && widths[d] < natural[d] && widths[d] >= DONOR_FLOOR + gap)
            .max_by_key(|&d| widths[d]);
        if let Some(d) = donor {
            widths[d] -= gap;
            widths[c] += gap;
        }
    }
}

/// Largest `level` in [lo, hi] with sum(level) <= budget (sum is monotonic).
fn binary_search_level(lo: usize, hi: usize, budget: usize, sum: &dyn Fn(usize) -> usize) -> usize {
    let (mut lo, mut hi) = (lo, hi);
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        if sum(mid) <= budget {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    lo
}

/// Hand out remaining budget one column at a time, neediest column first.
fn distribute_leftover(widths: &mut [usize], target: &[usize], budget: usize) {
    let mut used: usize = widths.iter().sum();
    while used < budget {
        let Some(c) = (0..widths.len())
            .filter(|&c| widths[c] < target[c])
            .max_by_key(|&c| target[c] - widths[c])
        else {
            break;
        };
        widths[c] += 1;
        used += 1;
    }
}

fn longest_word(spans: &[Span<'static>]) -> usize {
    // Approximate: longest whitespace-delimited token within each span.
    // (A word spanning styled fragments is rare inside table cells.)
    spans
        .iter()
        .flat_map(|s| s.content.split_whitespace())
        .map(crate::render::wrap::display_width)
        .max()
        .unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::model::Block;
    use crate::markdown::parser::parse;
    use crate::render::theme::{CharSet, Theme};
    use crate::render::wrap::display_width;

    fn render_table(src: &str, width: usize) -> Vec<String> {
        let theme = Theme::dark(CharSet::unicode());
        let links = LinkRegistry::new();
        let tr = TableRenderer::new(&theme, &links);
        match parse(src).blocks.into_iter().next() {
            Some(Block::Table {
                alignments,
                head,
                rows,
            }) => tr
                .render(&alignments, &head, &rows, width)
                .iter()
                .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
                .collect(),
            other => panic!("expected table, got {other:?}"),
        }
    }

    #[test]
    fn small_table_uses_natural_widths() {
        let lines = render_table("| A | B |\n|---|---|\n| one | two |\n", 80);
        assert!(lines[0].starts_with('╭'));
        assert!(lines.iter().all(|l| display_width(l) <= 80));
        // Header, one body row, three borders.
        assert_eq!(lines.len(), 5);
    }

    #[test]
    fn never_exceeds_width_even_when_squeezed() {
        // 10-column table at 60 cols: minimums overflow, must shrink.
        let mut src = String::from("| c1 | c2 | c3 | c4 | c5 | c6 | c7 | c8 | c9 | c10 |\n");
        src.push_str("|---|---|---|---|---|---|---|---|---|---|\n");
        src.push_str("| longvalue1 | longvalue2 | longvalue3 | longvalue4 | longvalue5 | longvalue6 | longvalue7 | longvalue8 | longvalue9 | longvalue10 |\n");
        let lines = render_table(&src, 60);
        for l in &lines {
            assert!(
                display_width(l) <= 60,
                "line {} wide: {l}",
                display_width(l)
            );
        }
    }

    #[test]
    fn numeric_columns_right_align() {
        let src = "| name | count |\n|---|---|\n| alpha | 1 |\n| beta | 22 |\n| gamma | 333 |\n";
        let lines = render_table(src, 80);
        // The numeric cell must sit flush against the right border
        // (column width = len("count") = 5).
        let row = lines.iter().find(|l| l.contains("alpha")).unwrap();
        assert!(
            row.ends_with("    1 │"),
            "expected right-aligned 1 in {row:?}"
        );
    }

    #[test]
    fn wrapped_cells_get_row_separators() {
        let src = "| a | text |\n|---|---|\n| 1 | this is a very long sentence that will surely wrap |\n| 2 | short |\n";
        let lines = render_table(src, 30);
        let separators = lines.iter().filter(|l| l.starts_with('├')).count();
        // one after header + one between the two body rows
        assert_eq!(separators, 2);
        for l in &lines {
            assert!(display_width(l) <= 30, "too wide: {l}");
        }
    }

    #[test]
    fn ragged_rows_are_padded() {
        let src = "| a | b | c |\n|---|---|---|\n| 1 |\n| 1 | 2 | 3 | 4 |\n";
        let lines = render_table(src, 40);
        assert!(!lines.is_empty());
        for l in &lines {
            assert!(display_width(l) <= 40);
        }
    }

    #[test]
    fn single_char_columns_fit_narrow_width() {
        // Regression: fake minimum budgets emitted 37-wide tables at w=20.
        let src = "| a | b | c | d | e | f | g | h | i |\n|---|---|---|---|---|---|---|---|---|\n| 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |\n";
        let lines = render_table(src, 20);
        assert!(lines.iter().any(|l| l.contains('│')), "expected a grid");
        for l in &lines {
            assert!(
                display_width(l) <= 20,
                "line {} wide: {l}",
                display_width(l)
            );
        }
    }

    #[test]
    fn cjk_cells_never_shear_borders() {
        // Width-2 graphemes cannot fit width-1 columns: must degrade to
        // records rather than emit rows wider than their borders.
        let src = "| 你 | 好 | 世 | 界 | 你 | 好 | 世 | 界 |\n|---|---|---|---|---|---|---|---|\n| 你 | 好 | 世 | 界 | 你 | 好 | 世 | 界 |\n";
        let lines = render_table(src, 20);
        for l in &lines {
            assert!(display_width(l) <= 20, "too wide: {l}");
        }
        // All lines of any grid must share one width; records trivially pass.
        if lines.iter().any(|l| l.contains('│')) {
            let widths: Vec<usize> = lines.iter().map(|l| display_width(l)).collect();
            assert!(
                widths.windows(2).all(|w| w[0] == w[1]),
                "sheared: {widths:?}"
            );
        }
    }

    #[test]
    fn header_only_table_has_no_phantom_row() {
        let src = "| Model | Size |\n|---|---|\n";
        let lines = render_table(src, 40);
        assert_eq!(lines.len(), 3, "top, header, bottom only: {lines:?}");
        assert!(!lines.iter().any(|l| l.starts_with('├')));
    }

    #[test]
    fn date_columns_stay_left_aligned() {
        let src = "| when | what |\n|---|---|\n| 2026-07-13 | a |\n| 2026-07-14 | b |\n";
        let lines = render_table(src, 40);
        let row = lines.iter().find(|l| l.contains("2026-07-13")).unwrap();
        assert!(
            row.contains("│ 2026-07-13 │"),
            "date should hug the left border: {row:?}"
        );
    }

    #[test]
    fn impossible_grid_degrades_to_records() {
        // 15 columns at width 20: a grid cannot exist (needs >= 31 cols).
        let head: Vec<String> = (1..=15).map(|i| format!("h{i}")).collect();
        let vals: Vec<String> = (1..=15).map(|i| format!("v{i}")).collect();
        let src = format!(
            "| {} |\n|{}\n| {} |\n",
            head.join(" | "),
            "---|".repeat(15),
            vals.join(" | ")
        );
        let lines = render_table(&src, 20);
        for l in &lines {
            assert!(display_width(l) <= 20, "record line too wide: {l}");
        }
        // All values survive.
        let all: String = lines.join("\n");
        assert!(all.contains("h15: v15"), "missing record content:\n{all}");
    }
}
