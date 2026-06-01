# Phase 1: 视野控制命令集

> 6 个核心命令（inspect/overview/tile/viewport/resize/rotate），给 AI Agent 用的本地图片视野控制。JSON 输出 + 坐标映射 + Agent-safe。

## 共享决策

| # | 决策 | 选择 | 详情 |
|---|------|------|------|
| FD1 | Workspace 结构 | vistools-core (lib) + vistools (bin) | core 导出公共 API，bin 只做 clap 解析 + 调 core |
| FD2 | 坐标映射策略 | 每个操作输出 coordinate_mapping | 包含 origin、scale、formula，Agent 可反向映射到源图 |
| FD3 | 输出格式 | 全部 CommandResult<T> JSON | ok/error/warnings/operation/data 结构统一 |
| FD4 | 文件安全校验 | 集中在 core::guard 模块 | 路径 sandbox + 像素限制 + 源文件保护 + 格式推断 |
| FD5 | tile 余数策略 | 最后一个 tile 包含余数像素 | 所有 tile 无缝覆盖完整源图，无遗漏 |
| FD6 | 输出格式推断 | 根据输出文件扩展名推断 | .png→PNG, .jpg/.jpeg→JPEG, .webp→WebP |
| FD7 | inspect 策略建议 | 基于 1568px 阈值 | 源图长边 > 1568px 时建议 needs_overview=true |

### FD1: Workspace 结构

**选择**：Cargo workspace，两个 crate
**理由**：core 可独立测试，未来 MCP server / 远端 AI 可复用 core。CLI binary 保持轻量。
**拒绝**：单 crate（core 和 CLI 耦合）、多 crate 过细（overhead > 收益）

```
image-viewport/
├── Cargo.toml              # workspace root
├── crates/
│   ├── core/
│   │   ├── Cargo.toml      # image-viewport-core
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs     # Point/Rect/Percent/Anchor/Size/CommandResult
│   │       ├── guard.rs     # Agent-safe 校验
│   │       ├── coord.rs     # 坐标计算 + 映射
│   │       ├── inspect.rs
│   │       ├── overview.rs
│   │       ├── tile.rs
│   │       ├── viewport.rs
│   │       ├── resize.rs
│   │       └── rotate.rs
│   └── cli/
│       ├── Cargo.toml       # image-viewport (depends on core)
│       └── src/
│           └── main.rs      # clap derive + 调 core
├── tests/                   # 集成测试
├── fixtures/                # 测试图片
└── docs/
```

### FD2: 坐标映射策略

**选择**：每个操作输出 `coordinate_mapping` 结构体
**理由**：Agent 在局部图里发现目标后，需要知道它在源图的坐标（computer-use 点击、UI 定位）。
**拒绝**：无映射（Agent 丢失位置信息）、单独 map-coords 命令（imgctl 方案，需额外调用）

### FD3: 输出格式

**选择**：统一 `CommandResult<T>` 泛型
**理由**：Agent 可用同一套解析逻辑处理所有命令输出。ok 时取 data，error 时取 error.code。
**拒绝**：TSV（imgctl 方案，需额外解析）

### FD4: 文件安全校验

**选择**：集中 `guard` 模块，所有命令入口调用
**理由**：校验逻辑复用（路径 sandbox + 像素限制 + 源文件保护），不散落在各命令中。
**拒绝**：各命令自行校验（重复代码、易遗漏）

---

## 共享数据模型

### 核心类型

```rust
/// 像素坐标点
struct Point { x: u32, y: u32 }

/// 像素矩形区域
struct Rect { x: u32, y: u32, width: u32, height: u32 }

/// 百分比坐标（0.0 - 1.0）
struct Percent { x: f64, y: f64, w: f64, h: f64 }

/// 九宫格锚点
enum Anchor {
    TopLeft, Top, TopRight,
    Left, Center, Right,
    BottomLeft, Bottom, BottomRight,
}

/// 图片尺寸
struct Size { width: u32, height: u32 }

/// 统一输出结构
struct CommandResult<T> {
    ok: bool,
    operation: String,
    input: String,           // 输入文件路径
    data: Option<T>,         // ok 时有值
    error: Option<ErrorInfo>, // !ok 时有值
    warnings: Vec<String>,
    elapsed_ms: u64,
}

struct ErrorInfo {
    code: String,            // "FILE_NOT_FOUND" 等
    message: String,
}

/// 坐标映射
struct CoordinateMapping {
    crop_origin_in_source: [u32; 2],  // [x, y]
    scale_factor: Option<f64>,         // None for no-scale ops
    formula: String,                   // "crop_x = source_x - 4400"
}

/// 源图信息（所有命令通用）
struct SourceInfo {
    width: u32,
    height: u32,
    format: String,        // "png", "jpeg", "webp"
    size_bytes: u64,
}
```

### 命令输出类型

```rust
// inspect
struct InspectOutput {
    source: SourceInfo,
    suggestion: Suggestion,
}
struct Suggestion {
    needs_overview: bool,
    max_tile_rows: u32,
    max_tile_cols: u32,
}

// overview
struct OverviewOutput {
    output: String,         // 输出文件路径
    source: SourceInfo,
    result: Size,
    scale_factor: f64,
    coordinate_mapping: CoordinateMapping,
}

// tile
struct TileOutput {
    source: SourceInfo,
    rows: u32,
    cols: u32,
    tiles: Vec<TileInfo>,
}
struct TileInfo {
    path: String,
    row: u32,
    col: u32,
    width: u32,
    height: u32,
    source_region: Rect,
}

// viewport
struct ViewportOutput {
    output: String,
    source: SourceInfo,
    crop: CropInfo,
    result: Size,
    coordinate_mapping: CoordinateMapping,
}
struct CropInfo {
    mode: String,           // "anchor" | "percent" | "rect"
    region: Rect,           // 转换为像素后的裁剪区域
    params: serde_json::Value, // 原始参数（anchor名/百分比值/像素值）
}

// resize
struct ResizeOutput {
    output: String,
    source: SourceInfo,
    result: Size,
    scale_factor: f64,
    coordinate_mapping: CoordinateMapping,
}

// rotate
struct RotateOutput {
    output: String,
    source: SourceInfo,
    result: Size,
    degrees: u32,
    coordinate_mapping: CoordinateMapping,
}
```

---

## 共享约束

### 安全

- `guard::validate_input_path(path)` — 拒绝含 `..` 的路径，文件必须存在
- `guard::validate_output_path(path)` — 拒绝含 `..` 的路径，不能与输入路径相同
- `guard::validate_dimensions(width, height)` — 拒绝 > 100MP 的图片
- `guard::validate_tile_count(rows, cols)` — 拒绝 rows * cols > 64

### 性能

- inspect 只读 header（`image::image_dimensions()`），不加载全图
- overview/resize 使用 `image::imageops::thumbnail()` 用于快速预览（Lanczos3 用于精确输出）
- tile 使用 `DynamicImage::crop()` 逐块提取，不加载多次

### 兼容性

- 输入格式：PNG、JPEG、WebP、TIFF、BMP、GIF（image-rs 默认 feature）
- 输出格式：根据文件扩展名推断（.png→PNG、.jpg→JPEG、.webp→WebP）
- JPEG 输出 quality=95（可通过 `--quality` 覆盖）
- 错误输出始终为 JSON（即使非 `--json` 模式也输出结构化错误到 stderr）

---

## 技术选型

> 引用 project.md 已有选型。本 feature 无额外依赖。

| 层 | 选择 | 版本 | 理由 |
|---|------|------|------|
| 图片处理 | image-rs | 0.25.x | project.md 已选 |
| CLI | clap | 4.x | project.md 已选 |
| 序列化 | serde + serde_json | 1.x | CommandResult<T> JSON 输出 |

---

## 领域索引

> 纯 CLI 工具，无 API/DB/Frontend 领域。

| 领域 | 目录 | 状态 | 说明 |
|------|------|------|------|
| core | crates/core/src/ | v1.0 | 图片处理核心库（所有命令逻辑） |
| cli | crates/cli/src/ | v1.0 | CLI 入口（clap 解析 + 调 core） |
| testing | tests/ | v1.0 | 集成测试（assert_cmd） |

---

## 模块索引

| 模块 | 文件 | 说明 |
|------|------|------|
| types | crates/core/src/types.rs | Point/Rect/Percent/Anchor/Size/CommandResult/CoordinateMapping |
| guard | crates/core/src/guard.rs | Agent-safe 校验（路径/像素/源文件保护） |
| coord | crates/core/src/coord.rs | 坐标计算（anchor→rect、percent→rect、映射公式） |
| inspect | crates/core/src/inspect.rs | inspect 命令逻辑 |
| overview | crates/core/src/overview.rs | overview 命令逻辑 |
| tile | crates/core/src/tile.rs | tile 命令逻辑 |
| viewport | crates/core/src/viewport.rs | viewport 命令逻辑（anchor/percent/rect） |
| resize | crates/core/src/resize.rs | resize 命令逻辑 |
| rotate | crates/core/src/rotate.rs | rotate 命令逻辑 |
| main | crates/cli/src/main.rs | CLI 入口（clap derive + 分发到 core） |

---

## 代码映射

```
contract.md ──────────→ crates/core/src/types.rs   (所有共享类型)
                         crates/core/src/guard.rs   (安全校验)
                         crates/core/src/coord.rs   (坐标计算)

modules/<name>.md ────→ crates/core/src/<name>.rs  (命令逻辑)
                         crates/cli/src/main.rs     (CLI 参数定义)
                         tests/<name>_test.rs        (集成测试)
```

---

## 编排

### 入口文件

`crates/cli/src/main.rs` — CLI 入口，负责：
1. 解析 clap 参数
2. 调用 `guard::validate_*` 校验
3. 调用 `core::<command>::execute()` 执行
4. 序列化 `CommandResult<T>` 为 JSON 输出到 stdout

### 调用链

```
main.rs
  ├─ clap parse → Command enum
  ├─ guard::validate_input_path(input)
  ├─ guard::validate_output_path(output)  (if has output)
  ├─ match command:
  │   ├─ Inspect → core::inspect::execute(input)
  │   ├─ Overview → core::overview::execute(input, output, max_width)
  │   ├─ Tile → core::tile::execute(input, rows, cols, out_dir)
  │   ├─ ViewportAnchor → coord::anchor_to_rect(anchor, w, h, source_size)
  │   │                  → core::viewport::execute(input, output, rect)
  │   ├─ ViewportPercent → coord::percent_to_rect(pct, source_size)
  │   │                   → core::viewport::execute(input, output, rect)
  │   ├─ ViewportRect → core::viewport::execute(input, output, rect)
  │   ├─ Resize → core::resize::execute(input, output, width, height)
  │   └─ Rotate → core::rotate::execute(input, output, degrees)
  └─ serde_json::to_string_pretty(&result) → stdout
```

### 坐标计算模块（coord.rs）职责

```
coord::anchor_to_rect(anchor, viewport_w, viewport_h, source_size) → Rect
  根据 anchor 九宫格语义计算裁剪起始坐标

coord::percent_to_rect(percent, source_size) → Rect
  百分比转像素：px = pct.x * source.width

coord::make_mapping(source_rect, source_size, result_size) → CoordinateMapping
  生成 crop_origin_in_source + scale_factor + formula
```

### 错误码枚举

```rust
enum ErrorCode {
    FileNotFound,
    UnsupportedFormat,
    InvalidDimensions,
    InvalidCoordinates,
    InvalidParameters,
    OutputWriteError,
    PathEscape,
    OutputSameAsInput,
    PixelLimitExceeded,
}

impl ErrorCode {
    fn as_str(&self) -> &'static str {
        match self {
            FileNotFound => "FILE_NOT_FOUND",
            UnsupportedFormat => "UNSUPPORTED_FORMAT",
            InvalidDimensions => "INVALID_DIMENSIONS",
            InvalidCoordinates => "INVALID_COORDINATES",
            InvalidParameters => "INVALID_PARAMETERS",
            OutputWriteError => "OUTPUT_WRITE_ERROR",
            PathEscape => "PATH_ESCAPE",
            OutputSameAsInput => "OUTPUT_SAME_AS_INPUT",
            PixelLimitExceeded => "PIXEL_LIMIT_EXCEEDED",
        }
    }
}
```
