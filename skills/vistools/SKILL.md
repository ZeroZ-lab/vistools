---
name: vistools
description: Visually inspect, navigate, and crop images using vistools CLI. Use after modifying frontend code, analyzing screenshots, or working with large images.
argument-hint: "<image-path> [focus or action]"
arguments: [image, action]
allowed-tools: Bash(${CLAUDE_PLUGIN_ROOT}/skills/vistools/scripts/vistools *) Bash(jq *) Bash(sips *) Bash(which *) Read
---

# vistools — Visual Tools for AI Agents

You are analyzing an image using `vistools`. The image is: `$image`
$action

The `vistools` binary is bundled with this plugin. Always invoke it as:
```
${CLAUDE_PLUGIN_ROOT}/skills/vistools/scripts/vistools
```
This wrapper auto-detects your platform (macOS/Linux, arm64/x64) and runs the correct binary.

## Step 1: Inspect (always first)

```bash
${CLAUDE_PLUGIN_ROOT}/skills/vistools/scripts/vistools inspect "$image" | jq .
```

Read the JSON output:
- `data.source.width` × `data.source.height` — dimensions
- `data.source.format` — "png", "jpeg", etc.
- `data.suggestion.needs_overview` — `true` if long side > 1568px
- `data.suggestion.max_tile_rows` × `max_tile_cols` — recommended grid

## Step 2: Decide What To Do

**Small image** (`needs_overview: false`) → analyze directly, skip overview.

**Large image** (`needs_overview: true`) → generate overview first:
```bash
${CLAUDE_PLUGIN_ROOT}/skills/vistools/scripts/vistools overview "$image" /tmp/iv-overview.png --max-width 1200 | jq .
```
Note the `scale_factor` — divide overview coordinates by it to get source coordinates.

## Step 3: Navigate — Pick Your Strategy

### A. Grid (full coverage)
```bash
${CLAUDE_PLUGIN_ROOT}/skills/vistools/scripts/vistools tile "$image" --rows 2 --cols 3 --out-dir /tmp/iv-tiles
```
Output lists every tile with `source_region`. Tiles cover the full source seamlessly.
Last tile per row/col absorbs remainder pixels.

### B. Positional crop (anchor)
```bash
${CLAUDE_PLUGIN_ROOT}/skills/vistools/scripts/vistools viewport anchor "$image" /tmp/iv-crop.png \
    --anchor center --width 800 --height 600
```
Anchors: `top-left` | `top` | `top-right` | `left` | `center` | `right` | `bottom-left` | `bottom` | `bottom-right`

### C. Proportional crop (percent)
```bash
${CLAUDE_PLUGIN_ROOT}/skills/vistools/scripts/vistools viewport percent "$image" /tmp/iv-crop.png \
    --x 0.3 --y 0.3 --w 0.4 --h 0.4
```
Values are fractions 0.0–1.0 of the source.

### D. Exact pixel crop (rect)
```bash
${CLAUDE_PLUGIN_ROOT}/skills/vistools/scripts/vistools viewport rect "$image" /tmp/iv-crop.png \
    --x 100 --y 200 --width 800 --height 600
```
Must not exceed source bounds — check with `inspect` first.

### E. Resize
```bash
${CLAUDE_PLUGIN_ROOT}/skills/vistools/scripts/vistools resize "$image" /tmp/iv-resized.png --width 800              # proportional
${CLAUDE_PLUGIN_ROOT}/skills/vistools/scripts/vistools resize "$image" /tmp/iv-resized.png --width 512 --height 512  # forced exact
```

### F. Rotate
```bash
${CLAUDE_PLUGIN_ROOT}/skills/vistools/scripts/vistools rotate "$image" /tmp/iv-rotated.png --degrees 90   # 0, 90, 180, 270
```

## Step 4: Coordinate Back-Mapping

Every output includes `coordinate_mapping`:
```json
{
  "crop_origin_in_source": [960, 720],
  "scale_factor": null,
  "formula": "source_x = result_x + 960, source_y = result_y + 720"
}
```

**To map point (rx, ry) in output back to source:**
- Crop only (scale_factor is null): `source = (rx + origin_x, ry + origin_y)`
- With scale: `source = (rx / scale + origin_x, ry / scale + origin_y)`

**Practical use:** If you spot a UI bug at (150, 80) in the crop, and `crop_origin_in_source` is [960, 720], the bug is at **(1110, 800)** in the source image.

## Step 5: Drill Down (recursive)

Crop a crop to zoom in:
```bash
${CLAUDE_PLUGIN_ROOT}/skills/vistools/scripts/vistools viewport anchor /tmp/iv-crop1.png /tmp/iv-crop2.png \
    --anchor top-left --width 400 --height 300
```
Chain coordinate mappings to trace back to the original source.

## Error Handling

All errors return `{"ok": false, "error": {"code": "...", "message": "..."}}`. Process exits non-zero.

| Code | Meaning | Fix |
|------|---------|-----|
| `FILE_NOT_FOUND` | Image doesn't exist | Check path |
| `UNSUPPORTED_FORMAT` | Can't decode | Not a valid image file |
| `INVALID_COORDINATES` | Rect exceeds bounds | Check source dimensions with inspect |
| `INVALID_PARAMETERS` | Bad tile count / degrees / width | Tile max 64, degrees ∈ {0,90,180,270} |
| `OUTPUT_SAME_AS_INPUT` | Would overwrite source | Use a different output path |
| `PIXEL_LIMIT_EXCEEDED` | > 100 megapixels | Use overview to scale down first |
| `PATH_ESCAPE` | Path has `..` | Use absolute or relative without `..` |

## Output Format

Every command returns the same envelope on stdout:
```json
{
  "ok": true,
  "operation": "viewport",
  "input": "src.png",
  "data": { ... },
  "warnings": [],
  "elapsed_ms": 12
}
```

## Report Format

After analysis, report:
1. **Image profile**: dimensions, format, whether it needed scaling
2. **Navigation path**: which regions you explored and why
3. **Findings**: what you observed in each region
4. **Source coordinates**: where issues are in the original image (using coordinate_mapping)

## Setup (if binary not found for platform)

If the wrapper reports a missing binary, compile from source (needs Rust 1.88+):

```bash
git clone https://github.com/ZeroZ-lab/vistools && cd vistools && cargo build --release
cp target/release/vistools ${CLAUDE_PLUGIN_ROOT}/skills/vistools/scripts/vistools-$(uname -s | tr '[:upper:]' '[:lower:]' | sed 's/darwin/macos/')-$(uname -m | sed 's/x86_64/x64/;s/aarch64/arm64/')
```

After install, verify: `${CLAUDE_PLUGIN_ROOT}/skills/vistools/scripts/vistools --version`
