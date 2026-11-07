# ✦ Familiar Engine (`FamiliarCore`)

An autonomous, 100% native Rust browser engine designed for the **Familiar Browser**.

---

## 1. Architecture Overview

`FamiliarCore` replaces heavy, external multi-gigabyte Chromium runtimes with a modular, lightweight, and memory-safe Rust layout and rendering pipeline.

```
Raw HTML / URL
      │
      ▼
┌──────────────┐
│ HTML Parser  │  ──> Builds Document Object Model (DOM Tree)
└──────────────┘      & extracts embedded <style> blocks
      │
      ▼
┌──────────────┐
│  CSS Parser  │  ──> Parses Selectors, Cascade, Declarations & Units (px, em, rem, %)
└──────────────┘
      │
      ▼
┌──────────────┐
│  Style Tree  │  ──> Specificity Matching & Computed Style Resolution (Display, Colors, Font, Margins)
└──────────────┘
      │
      ▼
┌──────────────┐
│ Layout Engine│  ──> CSS Box Model: Margin, Border, Padding, Content Dimensions & Flow Position
└──────────────┘
      │
      ▼
┌──────────────┐
│ Display List │  ──> Generates drawing primitives (SolidColor, Border, Text, Image)
└──────────────┘
      │
      ▼
┌──────────────┐
│ Rasterizer   │  ──> Software Canvas Framebuffer (RGBA8888, Windows 32-bit BMP, Data URL)
└──────────────┘
      │
      ▼
 Familiar Shell  (Rendered directly into browser tabs or native window)
```

---

## 2. Engine Pipeline Modules

| Module | File | Purpose |
|---|---|---|
| **DOM** | `src/dom/` | Node, ElementData, Text, Comment, Attribute maps, and tree pretty-printer |
| **HTML Parser** | `src/html/` | Lexer, entity decoder (`&amp;`, `&lt;`, etc.), void tag recognition, embedded CSS extraction |
| **CSS Parser** | `src/css/` | Selectors (tag, ID, class), Color parser (Hex, RGB, RGBA, named), Units (`px`, `em`, `rem`, `%`), User-Agent default stylesheet |
| **Style Tree** | `src/style/` | Cascading order, selector specificity `(ID, Class, Tag)`, computed styles |
| **Layout** | `src/layout/` | Box model (`content`, `padding`, `border`, `margin`), block flow, auto-width resolution, inline text sizing |
| **Paint / Rasterizer**| `src/paint/` | Display list generator, in-memory RGBA framebuffer, built-in crisp bitmap font renderer, 32-bit BMP exporter |
| **Pipeline API** | `src/lib.rs` | Public `Engine::render(html, width, height) -> RenderOutput` |
| **CLI Runner** | `src/main.rs` | Standalone CLI benchmarker and frame renderer (`cargo run`) |

---

## 3. How Familiar Uses the Engine

1. **Integrated Rust Backend:**
   `familiar-engine` is imported as a direct dependency in `src-tauri/Cargo.toml`:
   ```toml
   [dependencies]
   familiar-engine = { path = "../engine" }
   ```

2. **IPC Command (`src-tauri/src/lib.rs`):**
   `familiar_engine_render(html, width, height)` executes the complete engine pipeline in Rust and returns:
   - `data_url`: Base64 BMP image stream ready for canvas/tab rendering.
   - `dom_tree`: Formatted DOM hierarchy.
   - `command_count`: Number of display list draw commands.
   - `render_time_ms`: Exact rasterization time in milliseconds.

3. **Interactive Workbench in Familiar:**
   - Navigate to `familiar://engine` in Familiar (or open **Settings > Native Browser Engine**).
   - Write or edit HTML/CSS on the left and see it rendered live by the native Rust engine on the right.

---

## 4. Running the Engine Standalone

You can test and benchmark the engine independently from the command line:

```bash
cd d:\Familiar\engine

# Run built-in demo benchmark
cargo run

# Render any custom HTML file
cargo run -- path/to/page.html

# Render inline HTML snippet
cargo run -- "<div style='background: red; padding: 20px;'><h1>Hello!</h1></div>"
```
The engine outputs `familiar_rendered_frame.bmp` containing the rendered frame.
