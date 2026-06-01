# Test Cases — Phase 1: 视野控制命令集

> 从 PRD 验收条件推导的完整测试用例。覆盖正常、边界、错误三条路径。

## 测试范围矩阵

| 模块 | 单元测试 | 集成测试 | TDD | 覆盖程度 |
|------|---------|---------|-----|---------|
| types | ✅ 序列化/反序列化 | — | 否 | 基础 |
| guard | ✅ 校验逻辑 | ✅ CLI 错误输出 | 是 | **全覆盖** |
| coord | ✅ 坐标计算 | — | 是 | **全覆盖** |
| inspect | ✅ execute 函数 | ✅ CLI + JSON schema | 是 | **全覆盖** |
| overview | ✅ execute 函数 | ✅ CLI + 缩放验证 | 是 | **全覆盖** |
| tile | ✅ execute 函数 + 余数 | ✅ CLI + 文件生成 | 是 | **全覆盖** |
| viewport | ✅ 三种模式 | ✅ CLI + 裁剪验证 | 是 | **全覆盖** |
| resize | ✅ execute 函数 | ✅ CLI + 尺寸验证 | 是 | 正常+错误 |
| rotate | ✅ execute 函数 | ✅ CLI + 旋转验证 | 是 | 正常+错误 |
| main (CLI) | — | ✅ 全量 E2E | 否 | 冒烟 |

---

## guard 测试用例

### 正常

| 测试名 | 输入 | 预期 |
|--------|------|------|
| accept_valid_png | `"screenshot.png"` (exists) | Ok |
| accept_output_path | `"output/crop.png"` | Ok |
| accept_100mp_limit | `10000 x 10000` | Ok（刚好在限制内） |
| accept_tile_8x8 | `rows=8, cols=8` (64) | Ok |

### 边界

| 测试名 | 输入 | 预期 |
|--------|------|------|
| exact_pixel_limit | `10000 x 10000` | Ok |
| exact_tile_limit | `rows=8, cols=8` (64) | Ok |

### 错误

| 测试名 | 输入 | 预期 |
|--------|------|------|
| reject_path_traversal | `"../etc/passwd"` | `PATH_ESCAPE` |
| reject_nested_traversal | `"foo/../../bar"` | `PATH_ESCAPE` |
| reject_pixel_limit | `10001 x 10001` | `PIXEL_LIMIT_EXCEEDED` |
| reject_tile_count | `rows=10, cols=10` (100) | `INVALID_PARAMETERS` |
| reject_output_equals_input | same path | `OUTPUT_SAME_AS_INPUT` |
| reject_missing_file | `"nope.png"` | `FILE_NOT_FOUND` |
| reject_zero_dimensions | `0 x 100` | `INVALID_DIMENSIONS` |

---

## coord 测试用例

### anchor_to_rect（9 个方位全覆盖）

| 测试名 | anchor | viewport | source | 预期 Rect |
|--------|--------|----------|--------|-----------|
| top_left | TopLeft | 200x200 | 600x400 | (0, 0, 200, 200) |
| top | Top | 200x200 | 600x400 | (200, 0, 200, 200) |
| top_right | TopRight | 200x200 | 600x400 | (400, 0, 200, 200) |
| left | Left | 200x200 | 600x400 | (0, 100, 200, 200) |
| center | Center | 200x200 | 600x400 | (200, 100, 200, 200) |
| right | Right | 200x200 | 600x400 | (400, 100, 200, 200) |
| bottom_left | BottomLeft | 200x200 | 600x400 | (0, 200, 200, 200) |
| bottom | Bottom | 200x200 | 600x400 | (200, 200, 200, 200) |
| bottom_right | BottomRight | 200x200 | 600x400 | (400, 200, 200, 200) |

### percent_to_rect

| 测试名 | percent | source | 预期 Rect |
|--------|---------|--------|-----------|
| top_half | (0, 0, 1, 0.5) | 1000x1000 | (0, 0, 1000, 500) |
| center_quarter | (0.25, 0.25, 0.5, 0.5) | 1000x1000 | (250, 250, 500, 500) |
| bottom_strip | (0, 0.9, 1, 0.1) | 6000x4000 | (0, 3600, 6000, 400) |
| full_image | (0, 0, 1, 1) | 500x500 | (0, 0, 500, 500) |

### make_mapping

| 测试名 | origin | scale | 预期 formula |
|--------|--------|-------|-------------|
| crop_only | (4000, 0) | None | `"crop_x = source_x - 4000; crop_y = source_y"` |
| scaled | (0, 0) | 0.2 | `"overview_x = source_x * 0.2; overview_y = source_y * 0.2"` |

---

## inspect 测试用例

### 正常

| 测试名 | 输入 | 预期 |
|--------|------|------|
| large_png | 6000x4000 PNG | `width:6000, height:4000, needs_overview:true, max_tile_rows:4, max_tile_cols:3` |
| small_png | 200x150 PNG | `needs_overview:false` |
| jpeg_format | test.jpg | `format:"jpeg"` |
| webp_format | test.webp | `format:"webp"` |

### 边界

| 测试名 | 输入 | 预期 |
|--------|------|------|
| exactly_1568_wide | 1568x1000 | `needs_overview:false`（刚好不触发） |
| 1569_wide | 1569x1000 | `needs_overview:true` |

### 错误

| 测试名 | 输入 | 预期 |
|--------|------|------|
| missing_file | `"missing.png"` | `FILE_NOT_FOUND` |
| unsupported_format | `"test.xyz"` | `UNSUPPORTED_FORMAT` |
| corrupted_image | 空文件重命名为 .png | `UNSUPPORTED_FORMAT` |

---

## overview 测试用例

### 正常

| 测试名 | 输入 | 预期 |
|--------|------|------|
| scale_down | 6000x4000, max_width=1200 | 输出 1200x800, scale=0.2 |
| scale_to_jpeg | input.png, output.jpg | 输出 JPEG, format 推断正确 |
| coordinate_mapping | 6000x4000 → 1200x800 | `scale:0.2, formula 含 0.2` |

### 边界

| 测试名 | 输入 | 预期 |
|--------|------|------|
| no_upscale | 500px 宽, max_width=1000 | 原尺寸复制, warning |
| max_width_equals_source | 1200px 宽, max_width=1200 | 原尺寸复制, warning |
| very_small_max_width | 6000x4000, max_width=1 | 输出 1x1（极小但合法） |

### 错误

| 测试名 | 输入 | 预期 |
|--------|------|------|
| same_path | input==output | `OUTPUT_SAME_AS_INPUT` |
| output_dir_not_exists | `"/no/such/dir/out.png"` | `OUTPUT_WRITE_ERROR` |

---

## tile 测试用例

### 正常

| 测试名 | 输入 | 预期 |
|--------|------|------|
| 4x3_grid | 6000x4000, 4r 3c | 12 tiles, 每个 ~2000x1333 |
| 2x2_grid | 1000x1000, 2r 2c | 4 tiles, 每个 500x500 |
| 1x1_grid | 1000x1000, 1r 1c | 1 tile = 原图 |

### 边界（余数）

| 测试名 | 输入 | 预期 |
|--------|------|------|
| remainder_cols | 5000x4000, 3c | col0=1666, col1=1666, col2=1668 |
| remainder_rows | 4000x5000, 3r | row2 含余数 |
| full_coverage | 任意尺寸 | sum(tile widths) = source width, sum(tile heights) = source height |

### 错误

| 测试名 | 输入 | 预期 |
|--------|------|------|
| exceed_limit | 10x10 (100) | `INVALID_PARAMETERS` |
| zero_rows | 0r 3c | `INVALID_PARAMETERS` |
| out_dir_escape | `--out-dir ../escape` | `PATH_ESCAPE` |

---

## viewport 测试用例

### anchor 模式

| 测试名 | 输入 | 预期 Rect |
|--------|------|-----------|
| right_full_height | anchor=right, 2000x4000, 6000x4000 | (4000, 0, 2000, 4000) |
| bottom_strip | anchor=bottom, 6000x500, 6000x4000 | (0, 3500, 6000, 500) |
| center_square | anchor=center, 1000x1000, 3000x2000 | (1000, 500, 1000, 1000) |
| top_left_corner | anchor=top-left, 500x500, 2000x2000 | (0, 0, 500, 500) |

### percent 模式

| 测试名 | 输入 | 预期 Rect |
|--------|------|-----------|
| hero_section | (0, 0.1, 1, 0.3), 6000x4000 | (0, 400, 6000, 1200) |
| tiny_corner | (0.9, 0.9, 0.1, 0.1), 1000x1000 | (900, 900, 100, 100) |
| full_image | (0, 0, 1, 1), 500x500 | (0, 0, 500, 500) |

### rect 模式

| 测试名 | 输入 | 预期 Rect |
|--------|------|-----------|
| exact_crop | (4000, 2800, 2000, 1200) | 精确裁剪 |
| top_left_100x100 | (0, 0, 100, 100) | 精确裁剪 |

### 坐标映射

| 测试名 | 裁剪 | 预期 crop_origin_in_source |
|--------|------|---------------------------|
| anchor_mapping | right, 6000x4000 | [4000, 0] |
| percent_mapping | (0.5, 0.5, 0.5, 0.5), 1000x1000 | [500, 500] |
| rect_mapping | (300, 200, 400, 300) | [300, 200] |

### 错误

| 测试名 | 输入 | 预期 |
|--------|------|------|
| out_of_bounds | (5000, 0, 2000, 4000) in 6000x4000 | `INVALID_COORDINATES` |
| zero_width | width=0 | `INVALID_DIMENSIONS` |
| zero_height | height=0 | `INVALID_DIMENSIONS` |
| negative_percent | x=-0.1 | `INVALID_PARAMETERS` |
| over_100_percent | w=1.5 | `INVALID_PARAMETERS` |

---

## resize 测试用例（P1）

### 正常

| 测试名 | 输入 | 预期 |
|--------|------|------|
| proportional | 6000x4000, --width 1568 | 1568x1045 (等比例) |
| forced | 6000x4000, --width 800 --height 600 | 800x600 (非等比例) |
| scale_factor | 6000x4000, --width 1200 | scale_factor=0.2 |

### 边界

| 测试名 | 输入 | 预期 |
|--------|------|------|
| width_only | 1000x500, --width 500 | 500x250 (等比例) |
| same_size | 1000x500, --width 1000 | 原尺寸复制, warning |

### 错误

| 测试名 | 输入 | 预期 |
|--------|------|------|
| no_width | 不传 --width | `INVALID_PARAMETERS` |
| zero_width | --width 0 | `INVALID_PARAMETERS` |

---

## rotate 测试用例（P1）

### 正常

| 测试名 | 输入 | 预期 |
|--------|------|------|
| rotate_90 | 6000x4000, --degrees 90 | 输出 4000x6000 |
| rotate_180 | 6000x4000, --degrees 180 | 输出 6000x4000 |
| rotate_270 | 6000x4000, --degrees 270 | 输出 4000x6000 |
| coordinate_mapping | 90° | formula 含旋转坐标变换 |

### 边界

| 测试名 | 输入 | 预期 |
|--------|------|------|
| zero_degrees | --degrees 0 | 原尺寸复制, warning |
| same_dimensions_180 | 正方形 1000x1000, 180° | 输出 1000x1000 |

### 错误

| 测试名 | 输入 | 预期 |
|--------|------|------|
| invalid_degrees | --degrees 45 | `INVALID_PARAMETERS` |
| negative_degrees | --degrees -90 | `INVALID_PARAMETERS` |

---

## 集成测试矩阵（全量 E2E）

每个命令至少一次 CLI 调用 + JSON 输出验证：

| 命令 | CLI 调用 | 验证 |
|------|---------|------|
| inspect | `image-viewport inspect fixtures/256x256.png --json` | JSON ok=true, source.width=256 |
| overview | `image-viewport overview fixtures/256x256.png out.png --max-width 128 --json` | out.png 存在, result.width=128 |
| tile | `image-viewport tile fixtures/1000x1000.png --rows 2 --cols 2 --out-dir /tmp/t --json` | 4 个文件存在 |
| viewport anchor | `image-viewport viewport anchor fixtures/1000x1000.png /tmp/v.png --anchor right --width 500 --height 1000 --json` | v.png 存在, crop 正确 |
| viewport percent | `image-viewport viewport percent fixtures/1000x1000.png /tmp/v2.png --x 0.5 --y 0.5 --w 0.5 --h 0.5 --json` | v2.png 500x500 |
| viewport rect | `image-viewport viewport rect fixtures/1000x1000.png /tmp/v3.png --x 100 --y 100 --width 200 --height 200 --json` | v3.png 200x200 |
| resize | `image-viewport resize fixtures/256x256.png /tmp/r.png --width 128 --json` | r.png 128x128 |
| rotate | `image-viewport rotate fixtures/256x256.png /tmp/rt.png --degrees 90 --json` | rt.png 256x256 (正方形旋转不变) |

### 冒烟串联测试

```
inspect → overview → tile → viewport (完整 Agent 闭环模拟)
```
