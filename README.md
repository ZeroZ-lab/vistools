# vistools

[中文文档](README.zh-CN.md) | **English**

A programmable visual toolkit for AI agents. Inspect, navigate, and crop large images — every command returns structured JSON with coordinate mappings back to the source image.

```
$ vistools inspect screenshot.png
{
  "ok": true,
  "data": {
    "source": { "width": 3200, "height": 2400, "format": "png", "size_bytes": 808243 },
    "suggestion": { "needs_overview": true, "max_tile_rows": 2, "max_tile_cols": 3 }
  }
}
```

## Why

When an AI agent (Claude Code, Cursor, Codex, a browser agent) is handed a large screenshot or design file, it usually sees the whole thing at once — compressed, zoomed out, and too expensive to process at full resolution. `vistools` gives agents the same tools a human uses: look at the overview, pick a region of interest, zoom in, read the details.

Three design choices drive everything:

- **JSON-first.** Every command outputs a `CommandResult<T>` envelope — success or failure — so agents can parse the same shape every time.
- **Coordinate mapping.** Every crop, resize, and rotation includes a `coordinate_mapping` that describes how to translate output coordinates back to the source. An agent that finds a button in a crop knows exactly where it lives in the original.
- **Agent-safe.** The source file is never touched. Paths are sandboxed (no `..` escape). Pixel limits (100 MP) and tile limits (64) keep runaway calls from blowing up.

## Install

### Claude Code Plugin (recommended)

```bash
# In Claude Code:
/plugin install https://github.com/zhengjianqiao/vistools-skills
# Then: /vistools screenshot.png
```

### From source (Rust 1.88+)

```bash
git clone https://github.com/zhengjianqiao/vistolls
cd vistolls
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

When the long side exceeds 1568 px (Claude's visual-model threshold), `suggestion.needs_overview` is `true` and `max_tile_rows`/`max_tile_cols` tell you how fine a grid to use.

### `overview` — scaled-down preview

```bash
vistools overview large_screenshot.png overview.png --max-width 1200
```

Shrinks to fit `max_width`, preserves aspect ratio, returns the `scale_factor` so you can map clicks in the overview back to the source.

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

### `resize`

```bash
# Proportional (omit --height to preserve aspect ratio)
vistools resize src.png thumb.png --width 800

# Forced to exact dimensions
vistools resize src.png square.png --width 512 --height 512
```

### `rotate`

```bash
vistools rotate src.png rotated.png --degrees 90   # 0, 90, 180, 270
```

`--degrees 0` copies the file and emits a warning.

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
| `INVALID_COORDINATES` | Viewport rect exceeds source bounds |
| `INVALID_PARAMETERS` | Tile count > 64, degrees ∉ {0,90,180,270}, etc. |
| `OUTPUT_WRITE_ERROR` | Could not write the output file |
| `PATH_ESCAPE` | Path contains `..` |
| `OUTPUT_SAME_AS_INPUT` | Output would overwrite the source |
| `PIXEL_LIMIT_EXCEEDED` | Source exceeds 100 megapixels |

## Typical agent workflow

```
1. inspect src.png            # big image? what's the suggested grid?
       │
       ▼  needs_overview=true
2. overview src.png overview.png --max-width 1200
       │
       ▼  find region of interest in the overview
3a. tile src.png --rows 2 --cols 3 --out-dir ./tiles
       │
       ▼  or, if you know the area:
3b. viewport anchor src.png crop.png --anchor top-right --width 800 --height 600
       │
       ▼  coordinate_mapping tells you where (100, 50) in the crop lives in src.png
4. agent acts on the crop
```

The `coordinate_mapping.formula` string is the machine-readable recipe:

```
source_x = result_x + 2200, source_y = result_y          # crop
source_x = result_x / 0.375000                           # overview/resize
source_x = result_y, source_y = 2399 - result_x          # rotate 90°
```

## Skills

Skills are maintained in this repo (`skills/`) and auto-synced to [zhengjianqiao/vistools-skills](https://github.com/zhengjianqiao/vistools-skills) via GitHub Actions on every push to main.

```bash
# Claude Code — install from the skills-only repo (lightweight, no Rust source)
/plugin install https://github.com/zhengjianqiao/vistools-skills

# Then use: /vistools screenshot.png
```

Supports Claude Code, Cursor, and Codex. See `skills/README.md` for details.

### CI sync setup (repo maintainer)

The workflow at `.github/workflows/sync-skills.yml` copies `skills/` and `.claude-plugin/` to `vistools-skills` whenever those files change on main.

Required: a Personal Access Token with `repo` scope, added as `SKILLS_REPO_TOKEN` in this repo's Settings → Secrets and variables → Actions.

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
vistolls/
├── .claude-plugin/      # Claude Code plugin manifest
├── .github/workflows/   # CI: sync skills → vistools-skills
├── crates/
│   ├── core/            # library: types, guard, coord, one module per command
│   └── cli/             # thin clap wrapper + integration tests
├── fixtures/            # unit-test images (64x64, 256x256, 1000x1000)
│   └── e2e/             # real-world test images
├── skills/              # agent skill definitions (source of truth)
└── docs/                # design decisions (project.md), timeline, contracts
```

## License

MIT / Apache-2.0, at your option.

