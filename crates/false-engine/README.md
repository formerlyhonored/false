# false-engine

An experimental, self-contained HTML/CSS layout and paint workbench written in
Rust. It is used by the `false_engine_render` command in `false-ui` and can be
run from the command line.

It is **not** the engine behind normal browsing. False browses the web with
WebView2; this crate is a separate experiment that rasterizes a small subset of
HTML and CSS to a bitmap.

## What it does

```
HTML string
   -> HTML tokenizer + tree builder (dom/, html/)
   -> CSS parser (css/)
   -> style tree with specificity-based cascade (style/)
   -> block layout with the CSS box model (layout/)
   -> display list (paint/)
   -> RGBA framebuffer, BMP export (lib.rs)
```

| Module | Purpose |
|---|---|
| `dom/` | Node tree: elements, text, comments, attributes |
| `html/` | Tag/text/comment tokenizer, entity decoding, void elements, `<style>` extraction |
| `css/` | Stylesheet parser, simple selectors, hex/rgb/named colors, px/em/rem/% units, user-agent defaults |
| `style/` | Cascade and specificity `(id, class, tag)`, computed values |
| `layout/` | Block flow, box model, auto width, inline text sizing |
| `paint/` | Display list, software rasterizer, bitmap font, BMP output |
| `lib.rs` | `Engine::render(html, width, height) -> RenderOutput` |
| `main.rs` | CLI renderer for a demo page, a file, or an inline snippet |

## Known limits

- No JavaScript, no network loading, no images.
- No flexbox, grid, floats, or positioning beyond normal block/inline flow.
- Selectors cover tag, id, class, and descendant combinations only; no pseudo-classes.
- Fonts are a built-in bitmap face; there is no system font stack.
- The output is a single static frame, not a live document.

Treat it as a sandbox for layout and rasterization experiments, not a browser
engine roadmap.

## Usage

```bash
cargo run -p false-engine
cargo run -p false-engine -- path/to/page.html
cargo run -p false-engine -- "<div style='background:red;padding:20px'><h1>Hello</h1></div>"
```

The CLI writes `false_rendered_frame.bmp` in the working directory.

From the shell, the same pipeline is exposed through the `false_engine_render`
command, which returns a data URL, the DOM tree, the draw-command count, and the
render time.
