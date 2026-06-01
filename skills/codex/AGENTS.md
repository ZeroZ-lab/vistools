# vistools — Image Analysis Tool

When working with screenshots, design files, or any large image, use `vistools` to inspect, crop, and navigate images programmatically. All commands output structured JSON.

## Command Quick Reference

- `vistools inspect <path>` — metadata + strategy hint (needs_overview if > 1568px)
- `vistools overview <input> <output> --max-width <px>` — scaled-down preview
- `vistools tile <input> --rows <N> --cols <N> --out-dir <dir>` — grid split
- `vistools viewport anchor <input> <output> --anchor <pos> --width <px> --height <px>` — positional crop
- `vistools viewport percent <input> <output> --x <0-1> --y <0-1> --w <0-1> --h <0-1>` — percentage crop
- `vistools viewport rect <input> <output> --x <px> --y <px> --width <px> --height <px>` — pixel crop
- `vistools resize <input> <output> --width <px> [--height <px>]` — resize
- `vistools rotate <input> <output> --degrees <90|180|270>` — rotate

## Standard Workflow

1. Always `inspect` first — it's sub-millisecond and tells you the image size and whether to generate an overview.
2. If `needs_overview` is true, generate an overview at 1200px max-width. The `scale_factor` lets you map overview coordinates back to source.
3. Use `tile` for systematic full coverage or `viewport` for targeted region extraction.
4. Every output includes `coordinate_mapping` with `crop_origin_in_source` and `formula` — use these to convert output coordinates back to source image coordinates.

## Error Handling

Errors return `{"ok": false, "error": {"code": "...", "message": "..."}}`. Key codes: `FILE_NOT_FOUND`, `INVALID_COORDINATES`, `INVALID_PARAMETERS`, `OUTPUT_SAME_AS_INPUT`, `PIXEL_LIMIT_EXCEEDED`. Process exits non-zero on error.

## Constraints

- Never overwrite the source file (output path must differ from input).
- Max 100 megapixels input, max 64 tiles.
- Paths with `..` are rejected.
- Viewport rect must not exceed source bounds — use `inspect` to check dimensions first.
