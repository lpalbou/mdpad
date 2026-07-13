# Data flow

## Pipeline

```
                       ┌──────────────┐
  file / stdin ──────► │ parser       │  pulldown-cmark events -> Block tree
                       └──────┬───────┘
                              ▼
                       ┌──────────────┐
                       │ Vec<Block>   │  semantic model (markdown/model.rs)
                       └──────┬───────┘
                              ▼  width (re-runs on resize / toggle)
                       ┌──────────────┐
                       │ renderer     │  block.rs + table.rs + inline.rs
                       │              │  + wrap.rs + highlight.rs + theme.rs
                       └──────┬───────┘
                              ▼
                    ┌───────────────────┐
                    │ Vec<RenderedLine> │  1 line == 1 terminal row,
                    │  + HeadingAnchor  │  heading anchors mark TOC targets
                    └────┬─────────┬────┘
                         ▼         ▼
                   ┌─────────┐ ┌──────────┐
                   │ TUI     │ │ ANSI     │
                   │ viewer  │ │ print    │
                   └─────────┘ └──────────┘
```

## Inputs

- Markdown text (file path or stdin). Any UTF-8 content; malformed markdown
  parses to *something* rather than failing.
- Terminal width (or `--width`), theme flags, link-URL mode.

## Outputs

- TUI frames (ratatui buffer), or
- ANSI text (print mode): every line resets its style; `--color never`
  yields pure text.

## Table sizing (table.rs)

For each column compute:

- `natural` — widest unwrapped cell
- `minimum` — widest single word, capped at 16 (giant tokens hard-break)
- `header_w` — header width
- `typical` — 80th percentile of body cell widths

Then try stages until one fits the budget (width minus borders/padding);
stages whose fixed overhead alone exceeds the width are skipped:

| Stage | Padding | Floors (protected width per column) |
|---|---|---|
| A | 1 | `max(typical, header)` — headers whole, typical content whole |
| B | 1 | `min(floor_A, minimum)` — headers may word-wrap |
| C | 0 | same as A (padding sacrificed) |
| D | 0 | same as B |

If no stage fits — the word minimums overflow even unpadded — the table
renders as **records** instead: one `Header: value` list per row, separated
by short rules. A grid squeezed below word width is vertical confetti;
records keep every cell readable and are also the fallback for tables that
cannot physically exist at the width (15 columns at width 20).

Within a stage, spare width is distributed by max-min fairness ("water
filling"): every column gets its floor plus an equal share of the surplus,
capped at its natural width, remainder to the neediest columns. Finally,
columns 1–2 short of fitting entirely are topped up by shaving a wide column
that wraps anyway (`close_small_gaps`).

Columns with no declared alignment right-align when ≥ 70% of their non-empty,
non-placeholder body cells look numeric (digits with `.,%_` and a leading
sign only — internal dashes like dates stay left-aligned).

## View state (app.rs)

- `scroll` — index into rendered lines; clamped on every draw.
- Search: query -> byte ranges per line (smart-case); highlights are applied
  only to visible lines at draw time; matches recompute on re-render.
- TOC: heading anchors collected from rendered lines; Enter jumps to the
  anchor's line index.
- Editor: `tui-textarea` buffer of the raw source; on save the file is
  written, re-parsed and re-rendered; scroll clamps to the new line count.

## Mode transitions

```
View ──/──► SearchInput ──Enter/Esc──► View
View ──t──► Toc ──Enter/Esc──► View
View ──?──► Help ──Esc──► View
View ──e──► Edit ──Esc (clean)──► View
Edit ──Esc (dirty)──► ConfirmDiscard ──s/d──► View
                       └──────c──────► Edit
View ──E──► ($EDITOR subprocess) ──► View (reload)
```
