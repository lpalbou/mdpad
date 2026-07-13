# mdpad showcase

A stress document exercising every construct the renderer must handle
gracefully: headings, nested mixed lists, tables, code, quotes, links and
unicode. If this file looks good, most real files will.

## Text and inlines

Plain, **bold**, *italic*, ***bold italic***, ~~strikethrough~~, `inline code`,
a [named link](https://example.com/docs) and a bare one: <https://example.com>.
An image: ![terminal screenshot](assets/screen.png) and a footnote[^1].

A hard break follows this line.  
And this is after the break.

[^1]: Footnotes render at the definition site.

### Long words and URLs

Registries produce identifiers like
`LiquidAI/lfm2.5-1.2b-instruct:latest-with-a-very-long-suffix` and URLs like
https://internal.example.com/artifacts/build/2026-07-13/f00dfeedcafe/logs.txt
that must hard-break rather than overflow.

## Lists

- top level bullet
- another with **bold** and `code`
  - nested bullet with a longer sentence that will wrap when the terminal is
    narrow enough to force it
    - third level
  1. ordered inside unordered
  2. second item
     1. deeper ordered, roman-free
- [ ] an open task
- [x] a finished task

1. First ordered item
2. Second, with nested quote:
   > Quoted inside a list item.
3. Third

## Table: alignment and width

| Left | Center | Right |
|:-----|:------:|------:|
| a    | b      | c     |
| longer text here | mid | 42.5 |

## Table: numeric heuristic (no alignment markers)

| Model | Size | Gen tok/s | TTFT s | Done |
|---|---|---|---|---|
| gemma3:1b | 815 MB | 14.12 | 1.897 | stop |
| qwen3.5:0.8b | 1.0 GB | 8.03 | 1.773 | length |
| nomic-embed-text:latest | 274 MB | — | — | — |

## Code

```rust
/// Compute the nth Fibonacci number iteratively.
fn fib(n: u64) -> u64 {
    let (mut a, mut b) = (0u64, 1);
    for _ in 0..n {
        (a, b) = (b, a + b); // tuple swap keeps it branch-free
    }
    a
}
```

```
plain code block without language
tabs	are	expanded
```

Indented code:

    ls -la | grep total

## Quotes

> A single-level quote with enough text to wrap on narrow terminals and show
> the continuation bar clearly.
>
> > Nested quote, second level.
>
> Back to the first level, with `code` and **bold**.

## Unicode

CJK: 终端里的漂亮排版是这个工具存在的意义。 Emoji: 🚀 fits in tables too:

| 名前 | 説明 |
|---|---|
| 渲染 | 中文单元格内容 |
| 🚀 | emoji cell |

---

## HTML passthrough

<div class="banner">
  raw html renders dimmed, verbatim
</div>

The end.
