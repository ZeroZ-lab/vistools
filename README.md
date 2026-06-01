# vistools

> **A coordinate-grounded visual inspection protocol for AI coding agents.**

[中文文档](README.zh-CN.md) | **English**

A programmable visual toolkit for AI agents. Inspect, navigate, crop, and sample large images — every command returns structured JSON, and generated views include coordinate mappings back to the source image.

```
$ vistools inspect screenshot.png
{
  "ok": true,
  "data": {
    "source": { "width": 3200, "height": 2400, "format": "png", "size_bytes": 808243 },
    "suggestion": {
      "needs_overview": true,
      "max_tile_rows": 2,
      "max_tile_cols": 3,
      "recommended_next": "overview",
      "reason": "long side 3200 exceeds 1568 visual threshold",
      "suggested_max_side": 1568
    }
  }
}
```

## Before vs After

**Without vistools:**
```
Agent reads a 3200×2400 screenshot → details lost in compression
  → claims "the button looks correct" → no way to verify
```

**With vistools:**
```
1. inspect → 3200×2400, needs overview
2. overview --max-side 1200 → scaled preview, scale_factor = 0.375
3. Spot anomaly at overview (800, 600) → map to source: (2133, 1600)
4. viewport rect → exact crop of the region
5. sample --x 2133 --y 1600 → color is #e74c3c, not expected #2563eb
6. Report: "Button at source (2133, 1600) has incorrect background color"
```

## Why

When an AI agent (Claude Code, Cursor, Codex, a browser agent) is handed a large screenshot or design file, it usually sees the whole thing at once — compressed, zoomed out, and too expensive to process at full resolution. `vistools` gives agents the same tools a human uses: look at the overview, pick a region of interest, zoom in, read the details.

Three design choices drive everything:

- **JSON-first.** Every command outputs a `CommandResult<T>` envelope — success or failure — so agents can parse the same shape every time.
- **Coordinate mapping.** Every generated view includes a `coordinate_mapping` that describes how to translate output coordinates back to the source. An agent that finds a button in a crop knows exactly where it lives in the original.
- **Agent-safe.** The source file is never touched. Paths are sandboxed (no `..` escape). Pixel limits (100 MP) and tile limits (64) keep runaway calls from blowing up.

## Install

### Claude Code Plugin (recommended)

```bash
# In Claude Code:
/plugin install https://github.com/ZeroZ-lab/vistools-skills
# Then: /vistools screenshot.png
```

### From source (Rust 1.88+)

```bash
git clone https://github.com/ZeroZ-lab/vistools
cd vistools
cargo install --path crates/cli   # installs to ~/.cargo/bin/vistools
```

Or build and run directly:

```bash
cargo build --release
./target/release/vistools <command>
```

The release binary is a single ~5 MB executable with no runtime dependencies.

## Commands

### `inspect` — metadata + strategy hint

The first thing to call on any unknown image. Reads only the header, so it's sub-millisecond.

```bash
vistools inspect large_screenshot.png
```

When the long side exceeds 1568 px (Claude's visual-model threshold), `suggestion.recommended_next` is `overview`; otherwise it is `direct`. `max_tile_rows`/`max_tile_cols` tell you how fine a grid to use if you need full coverage.

### `overview` — scaled-down preview

```bash
vistools overview large_screenshot.png overview.png --max-side 1200
```

Shrinks so the longest side fits `max_side`, preserves aspect ratio, and returns the `scale_factor` so you can map clicks in the overview back to the source.

### `tile` — grid split

```bash
vistools tile large_screenshot.png --rows 2 --cols 3 --out-dir ./tiles
```

Produces `row-N-col-M.<ext>` files. The last tile in each row/column absorbs the remainder pixels, so the tiles always cover the source exactly.

### `viewport` — crop a region

Three modes, same output shape:

```bash
# Anchor-based (nine-position: top-left, center, bottom-right, ...)
vistools viewport anchor src.png crop.png --anchor top-right --width 800 --height 600

# Percentage-based (fractions of the source)
vistools viewport percent src.png crop.png --x 0.3 --y 0.3 --w 0.4 --h 0.4

# Pixel rectangle
vistools viewport rect src.png crop.png --x 1100 --y 200 --width 700 --height 700
```

Percent mode is strict: `x/y/w/h` must stay within `0..1`, and `x + w` / `y + h` must not exceed `1`.

### `sample` — point and region color picker

```bash
# Point color
vistools sample src.png --x 120 --y 80

# Average color and alpha stats for a region
vistools sample src.png --rect 100,80,40,40
```

Point mode returns `rgba`, `rgb`, lowercase `hex`, and `alpha`. Rect mode returns the rounded average color, `alpha_stats` (`min`, `max`, `average`, `transparent_ratio`), and `pixel_count`. `sample` is read-only and does not create an output image.

### Help & version

```bash
vistools --help              # list all commands with brief description
vistools inspect --help      # detailed help for a subcommand
vistools --version           # print version (e.g. "vistools 0.2.0")
```

## JSON output

Every command — success or failure — returns the same envelope on stdout:

```json
{
  "ok": true,
  "operation": "viewport",
  "input": "src.png",
  "data": {
    "output": "crop.png",
    "source": { "width": 3200, "height": 2400, "format": "png", "size_bytes": 808243 },
    "crop": {
      "mode": "anchor",
      "region": { "x": 2200, "y": 0, "width": 1000, "height": 600 },
      "params": { "anchor": "TopRight", "width": 1000, "height": 600 }
    },
    "result": { "width": 1000, "height": 600 },
    "coordinate_mapping": {
      "crop_origin_in_source": [2200, 0],
      "scale_factor": null,
      "formula": "source_x = result_x + 2200, source_y = result_y"
    }
  },
  "warnings": [],
  "elapsed_ms": 12
}
```

On failure, `ok` is `false`, `data` is absent, and `error` carries a stable machine-readable `code`:

```json
{
  "ok": false,
  "operation": "inspect",
  "input": "/tmp/nope.png",
  "error": { "code": "FILE_NOT_FOUND", "message": "input file not found: /tmp/nope.png" },
  "warnings": [],
  "elapsed_ms": 0
}
```

The process also exits non-zero on failure.

### Error codes

| Code | Meaning |
|------|---------|
| `FILE_NOT_FOUND` | Input file does not exist or is not a regular file |
| `UNSUPPORTED_FORMAT` | Image decoder could not read the file |
| `INVALID_DIMENSIONS` | Zero width/height passed to a command |
| `INVALID_COORDINATES` | Viewport/sample point or rect exceeds source bounds |
| `INVALID_PARAMETERS` | Tile count > 64, zero max side, malformed sample mode, etc. |
| `OUTPUT_WRITE_ERROR` | Could not write the output file |
| `PATH_ESCAPE` | Path contains `..` |
| `OUTPUT_SAME_AS_INPUT` | Output would overwrite the source |
| `PIXEL_LIMIT_EXCEEDED` | Source exceeds 100 megapixels |

## Typical agent workflow

```
1. inspect src.png            # big image? what's the suggested grid?
       │
       ▼  needs_overview=true
2. overview src.png overview.png --max-side 1200
       │
       ▼  find region of interest in the overview
3a. tile src.png --rows 2 --cols 3 --out-dir ./tiles
       │
       ▼  or, if you know the area:
3b. viewport anchor src.png crop.png --anchor top-right --width 800 --height 600
       │
       ▼  coordinate_mapping tells you where (100, 50) in the crop lives in src.png
4. sample src.png --x 1110 --y 800
       │
       ▼  inspect the exact color/alpha at the source coordinate
5. agent acts on the crop
```

The `coordinate_mapping.formula` string is the machine-readable recipe:

```
source_x = result_x + 2200, source_y = result_y          # crop
source_x = result_x / 0.375000, source_y = result_y / 0.375000   # overview
```

## Skills

Skills are maintained in a separate repo: [ZeroZ-lab/vistools-skills](https://github.com/ZeroZ-lab/vistools-skills).

```bash
# Claude Code — install from the skills-only repo
/plugin install https://github.com/ZeroZ-lab/vistools-skills

# Then use: /vistools screenshot.png
```

Supports Claude Code, Cursor, and Codex.

## Building

```bash
cargo build                       # debug
cargo build --release             # release (~5 MB, LTO + stripped)
cargo test                        # unit + integration
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Supported input formats: PNG, JPEG, WebP, TIFF, BMP, GIF.
Output format is inferred from the output file's extension.

## Project layout

```
vistools/
├── crates/
│   ├── core/            # library: types, guard, coord, one module per command
│   └── cli/             # thin clap wrapper + integration tests
├── fixtures/            # unit-test images (64x64, 256x256, 1000x1000)
│   └── e2e/             # real-world test images
└── docs/                # design decisions (project.md), timeline, contracts
```

## License

MIT / Apache-2.0, at your option.
