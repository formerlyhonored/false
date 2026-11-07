---
name: webgl-canvas-optimizer
description: >-
  Provides execution constraints, memory rules, and performance guidelines for
  high-framerate (60fps/120fps) WebGL, WebGPU, and 2D Canvas rendering loops.
  Use when writing, optimizing, or reviewing graphics rendering code, shaders,
  render pipelines, or canvas loops.
---

# WebGL & Canvas Optimizer Skill

Enforces high-performance constraints for WebGL, WebGPU, and 2D Canvas render pipelines. Targets consistent 60fps / 120fps frame rates by eliminating garbage collection spikes, minimizing GPU draw calls, and optimizing memory bandwidth.

---

## The Frame Budget Constraints

To achieve smooth rendering without jank:
- **60 FPS**: $16.6\text{ ms}$ total frame budget $\rightarrow$ Render loop CPU work must not exceed **$10\text{ ms}$**.
- **120 FPS**: $8.3\text{ ms}$ total frame budget $\rightarrow$ Render loop CPU work must not exceed **$5\text{ ms}$**.

Any CPU spikes over this budget will drop frames and cause visible stutter.

---

## Core Performance Rules

### 1. Zero Allocations in the Render Loop
Garbage Collection (GC) pauses are the #1 cause of framerate drops in canvas applications.
- **Rule**: Absolutely **no object, array, or closure allocations** inside the per-frame update/draw function (`requestAnimationFrame` or tick loop).
- **Techniques**:
  - Pre-allocate scratch vectors, matrices, and color objects once during initialization.
  - Re-use buffers and arrays by clearing or setting `.length = 0` rather than instantiating new arrays.
  - Avoid array functional methods (`.map()`, `.filter()`, `.forEach()`) in render loops; use indexed `for` loops or while loops.

```javascript
// ❌ BAD: Allocates 2 vectors and a callback closure every frame
function render() {
  const delta = new Vector2(dx, dy);
  entities.forEach(e => e.draw(delta));
  requestAnimationFrame(render);
}

// ✅ GOOD: Zero allocations per frame, pre-allocated scratch vector
const scratchDelta = new Vector2(0, 0);
function render() {
  scratchDelta.set(dx, dy);
  for (let i = 0, len = entities.length; i < len; i++) {
    entities[i].draw(scratchDelta);
  }
  requestAnimationFrame(render);
}
```

### 2. Batching Draw Calls
The overhead of a draw call (`gl.drawArrays`, `gl.drawElements`, `ctx.stroke`) is dominated by CPU-to-GPU synchronization:
- Group similar objects into a single vertex buffer and render with a single draw call.
- Use **Instanced Rendering** (`gl.drawElementsInstanced`, `ANGLE_instanced_arrays`, or WebGL2) for rendering multiples of the same geometry (e.g., particles, sprites, tiles, UI icons).
- Target fewer than **100 draw calls per frame** for complex scenes; fewer than **20** for UI/2D layers.

### 3. GPU State Change Minimization
Switching GPU pipeline state is costly. Sort render queues before drawing:
1. Sort by **Shader Program** (bind shader once).
2. Sort by **Texture / Material** (use Texture Atlases / Texture Arrays to avoid rebinding).
3. Sort by **Blend Mode / Depth State**.
4. Draw opaque geometry front-to-back (maximizes early-Z culling), followed by transparent geometry back-to-front.

### 4. Canvas & High-DPI Resolution Scaling
Uncapped High-DPI canvas resolutions cause fill-rate bottlenecks:
- Clamp `devicePixelRatio` to a maximum of 2.0 (rendering at 3.0+ on mobile retina screens wastes fill-rate with negligible visual benefit).
- Update canvas buffer dimensions (`canvas.width`, `canvas.height`) **only when the window or container resizes**, never inside the per-frame render loop.

```javascript
const dpr = Math.min(window.devicePixelRatio || 1, 2.0);
canvas.width = Math.floor(rect.width * dpr);
canvas.height = Math.floor(rect.height * dpr);
```

### 5. Resilient Context Lifecycle Management
WebGL contexts can be lost under GPU pressure or driver resets:
- Always register event listeners for `webglcontextlost` and `webglcontextrestored`.
- Cancel the animation frame loop on context loss and re-upload textures, shaders, and buffers on context restoration.

---

## Render Loop Audit Checklist

Before approving any rendering code:
- [ ] No `new` keywords or array allocations inside `render()` or `update()`.
- [ ] Draw calls are batched; no individual draw call per single sprite or primitive.
- [ ] Texture uploads (`texImage2D`, `texSubImage2D`) are executed during assets loading or background tasks, not synchronously mid-frame.
- [ ] OffscreenCanvas and Web Workers are utilized if heavy CPU rasterization or procedural geometry calculation is required.
- [ ] Frame time is instrumented with `performance.now()` or Chrome DevTools Performance Profiler.
