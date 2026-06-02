# 摄影计量: histogram --rgb / zone-map / exposure / focus-map / white-balance

> Agent 可用摄影语言级计量命令回答"曝光对不对"、"焦点在哪里"、"白平衡是否偏色"。命令全部只读，纯像素数学。

## 版本信息

| 项目 | 值 |
|------|-----|
| 日期 | 2026-06-02 |
| 阶段 | P1 详设（L2 stage） |
| 范围 | P0: histogram --rgb + zone-map + exposure；P1: focus-map + white-balance |

---

## 共享决策

| # | 决策 | 选择 | 详情 |
|---|------|------|------|
| FD1 | histogram RGB 输出策略 | --rgb flag 增量输出 | 不传时输出与现有完全一致；传时额外输出 r/g/b 三通道 |
| FD2 | Zone System 映射算法 | 线性 11 区 | luma 0-255 线性分 Zone 0-X，每区 ≈23 个灰度级 |
| FD3 | EV 计算公式 | log2(luma / 118) | 118 = sRGB 中灰（Zone V 中心值） |
| FD4 | 测光模式加权 | 4 种：evaluative / spot / center-weighted / highlight-weighted | 各模式独立加权函数 |
| FD5 | assessment 边界 | ±0.5 EV | ev < -0.5 → under，-0.5..0.5 → correct，> 0.5 → over |
| FD6 | CLI 参数模式 | 复用 RegionArgs + 新增 ExposureArgs | histogram/zone-map 复用现有 RegionArgs；exposure 新增 --mode/--x/--y |
| FD7 | focus-map 网格策略 | 复用 tile remainder 规则 | 最后一行/列吸收余数像素，cell.region 全部映射源图坐标 |
| FD8 | focus-map 锐度算法 | 复用 sharpness kernel | 每格输出 SharpnessMetrics，best_cell 取 score 最大值 |
| FD9 | white-balance 算法 | 灰世界通道增益估计 | mean_gray / channel_mean 生成 R/G/B gains，不输出修正图片 |
| FD10 | white-balance 精度边界 | 不输出 Kelvin | 只输出 warm/cool、green/magenta bias 和 gains，避免虚假色温精度 |

### FD1: histogram RGB 输出策略

**选择**：`--rgb` flag 增量输出
**理由**：向后兼容（PD9）。现有 histogram 的 `bins` 字段保留为亮度直方图，新增 `r`/`g`/`b` 字段为三通道数据。Agent 不传 `--rgb` 时行为完全不变。
**拒绝**：替换现有 bins（破坏兼容）、独立命令 `histogram-rgb`（命令面膨胀）

### FD2: Zone System 映射算法

**选择**：线性 11 区映射
```
zone_index = min(10, luma * 11 / 256)
Zone 0  = luma 0-22    (纯黑)
Zone I  = luma 23-46
Zone II = luma 47-69
Zone III= luma 70-92
Zone IV = luma 93-115
Zone V  = luma 116-139  (中灰，≈118)
Zone VI = luma 140-162
Zone VII= luma 163-185
Zone VIII=luma 186-208
Zone IX = luma 209-231
Zone X  = luma 232-255  (纯白)
```
**理由**：和现有 histogram 的 bins 一致（都是线性 0-255 映射），实现简单， photographers 理解直观。
**拒绝**：sRGB gamma 反转后映射（增加复杂度，且和 histogram 的线性 luma 不一致）、自适应分区（Zone System 的核心价值是固定参考系）

### FD3: EV 计算公式

**选择**：`EV = log2(weighted_mean_luma / 118.0)`
**理由**：118 是 sRGB 中灰亮度值，对应 Zone V 中心。EV = 0 表示"正确曝光"，正数过曝，负数欠曝。公式与摄影曝光补偿一致（+1 EV = 亮一档）。
**拒绝**：绝对 EV（需要 EXIF 推算场景亮度，超出纯像素范畴）、直方图面积加权积分（过度复杂）

### FD4: 测光模式加权

**选择**：4 种模式，各有独立加权函数
| 模式 | 加权函数 | 说明 |
|------|---------|------|
| evaluative | uniform | 全部像素等权均值 |
| spot | point(x, y) | 单像素亮度，必须提供 --x --y |
| center-weighted | Gaussian(σ = min(w,h)/3) | 中心高斯加权 |
| highlight-weighted | top 10% luma | 只取亮度前 10% 的像素均值 |

**理由**：与相机测光模式一一对应，摄影师一看就懂。
**拒绝**：矩阵测光（需要场景分割，超出纯像素范畴）

### FD5: assessment 边界

**选择**：±0.5 EV 为 correct 阈值
**理由**：0.5 EV 是摄影中"可接受曝光偏差"的常见标准（1/3 档精确，1/2 档可接受）。
**拒绝**：±0.3 EV（太严格，多数照片会被标记为 under/over）、±1.0 EV（太宽松）

### FD6: CLI 参数模式

**选择**：histogram/zone-map 复用 `RegionArgs`；exposure 新增 `ExposureArgs`（+ --mode/--x/--y）
**理由**：histogram 和 zone-map 只需 input + 可选 rect，和现有 photo 命令一致。exposure 需要额外的测光模式参数。
**拒绝**：所有新命令统一新 Args 类型（histogram/zone-map 无需额外参数）

### FD7: focus-map 网格策略

**选择**：复用 tile remainder 规则，最后一行/列吸收余数像素。
**理由**：和现有 tile/focus-map 实现一致，能保证所有 cell 合并后完整覆盖输入 region。
**拒绝**：均匀浮点切分后四舍五入（可能产生缝隙或重叠）、丢弃余数像素（破坏坐标完整性）

### FD8: focus-map 锐度算法

**选择**：复用现有 `sharpness` kernel。
**理由**：focus-map 是 sharpness 的网格化版本，复用算法能让整体 sharpness 和局部 sharpness 的含义一致。
**拒绝**：另起 Laplacian/FFT 算法（引入语义漂移且不符合最小 P1 范围）

### FD9: white-balance 算法

**选择**：灰世界通道增益估计。
```
mean_gray = (mean_r + mean_g + mean_b) / 3
gain_r = mean_gray / mean_r
gain_g = mean_gray / mean_g
gain_b = mean_gray / mean_b
```
**理由**：纯像素数学、无新增依赖、输出可直接被 Agent 消费为通道校正方向。
**拒绝**：学习式白平衡、RAW 白点估计、场景语义白点选择（都超出本地只读计量范围）

### FD10: white-balance 精度边界

**选择**：不输出 Kelvin 绝对色温，只输出方向性 bias。
**理由**：JPEG/RGB 像素缺少相机响应、RAW 白点和光源模型，Kelvin 数值容易产生虚假精度。Agent 需要的是偏色方向和校正倾向。
**拒绝**：直接估算 Kelvin（误导性强）、输出自然语言建议（由 Agent 基于 JSON 生成）

---

## 共享数据模型

> 协议类型定义在 `crates/core/src/protocol.rs`。

### 新增类型

```rust
// ─── histogram --rgb 增强 ───

// 现有 HistogramOutput 不变，新增可选字段：
// HistogramOutput.histogram 新增: rgb: Option<RgbHistogram>

pub struct RgbHistogram {
    pub r: ChannelHistogram,
    pub g: ChannelHistogram,
    pub b: ChannelHistogram,
}

pub struct ChannelHistogram {
    pub bins: Vec<u64>,          // 256 bins
    pub mean: f64,
    pub p05: u8,
    pub p50: u8,
    pub p95: u8,
    pub clipping_low: u64,      // luma ≤ 5 的像素数
    pub clipping_high: u64,     // luma ≥ 250 的像素数
}

// ─── zone-map ───

pub struct ZoneMapOutput {
    pub source: SourceInfo,
    pub region: Rect,
    pub zones: Vec<ZoneInfo>,   // 11 个，Zone 0 到 Zone X
}

pub struct ZoneInfo {
    pub zone: u8,                // 0-10 (0=X, 用数字)
    pub label: String,           // "0" / "I" / "II" ... "X"
    pub luma_range: (u8, u8),    // 该 zone 的亮度范围 (low, high)
    pub pixel_count: u64,
    pub ratio: f64,              // 占总像素比
    pub representative_rect: Rect,
}

// ─── exposure ───

pub struct ExposureOutput {
    pub source: SourceInfo,
    pub region: Rect,
    pub metering: String,               // "evaluative" / "spot" / "center_weighted" / "highlight_weighted"
    pub ev: f64,                         // 曝光值偏移
    pub assessment: String,             // "under" / "correct" / "over"
    pub mean_luma: f64,                 // 加权平均亮度
    pub spot_point: Option<Point>,      // spot 模式下的采样点
}

// ─── focus-map ───

pub struct FocusMapOutput {
    pub source: SourceInfo,
    pub region: Rect,
    pub rows: u32,
    pub cols: u32,
    pub cells: Vec<FocusCell>,
    pub best_cell: FocusCell,
    pub focus_point: Point,
}

pub struct FocusCell {
    pub row: u32,
    pub col: u32,
    pub region: Rect,
    pub sharpness: SharpnessMetrics,
}

// ─── white-balance ───

pub struct WhiteBalanceOutput {
    pub source: SourceInfo,
    pub region: Rect,
    pub white_balance: WhiteBalanceMetrics,
}

pub struct WhiteBalanceMetrics {
    pub rgb_mean: RgbMeans,
    pub gray_world_gains: RgbGains,
    pub temperature_bias: String, // "warm" / "neutral" / "cool"
    pub tint_bias: String,        // "green" / "neutral" / "magenta"
    pub assessment: String,       // "neutral" / "biased"
}

pub struct RgbMeans {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

pub struct RgbGains {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}
```

### 修改现有类型

```rust
// HistogramMetrics 新增可选 rgb 字段
pub struct HistogramMetrics {
    // 现有字段不变:
    pub bins: Vec<u64>,
    pub pixel_count: u64,
    pub mean_luma: f64,
    pub median_luma: u8,
    pub p05_luma: u8,
    pub p95_luma: u8,
    // 新增:
    pub rgb: Option<RgbHistogram>,  // 仅 --rgb 时有值
}
```

---

## 模块索引

| 模块 | 文件 | 公共接口 | 职责 | 新增/修改 |
|------|------|----------|------|----------|
| photo algorithms | crates/core/src/photo.rs | `execute_histogram(input, rect, rgb)`, `execute_zone_map(input, rect)`, `execute_exposure(input, rect, mode, spot_point)`, `execute_focus_map(input, rect, rows, cols)`, `execute_white_balance(input, rect)` | 摄影计量命令语义、区域加载、像素遍历、锐度/曝光/白平衡算法 | **修改** |
| protocol DTOs | crates/core/src/protocol.rs | `HistogramOutput`, `ZoneMapOutput`, `ExposureOutput`, `FocusMapOutput`, `WhiteBalanceOutput` | 对外 JSON schema 类型 | **修改** |
| CLI photo commands | crates/cli/src/commands/photo.rs | `RegionArgs`, `HistogramArgs`, `ExposureArgs`, `FocusMapArgs`, `run_*` | clap 参数、rect 解析、core 调用适配 | **修改** |
| CLI command registry | crates/cli/src/main.rs | `Commands::{Histogram, ZoneMap, Exposure, FocusMap, WhiteBalance}` | 顶层子命令注册和 dispatch | **修改** |

---

## 共享约束

> 引用 project.md：PD3（Agent-safe 只读）、PD4（稳定错误码）、PD7（纯像素数学）、PD8（photo.rs 内扩展）、PD9（histogram 向后兼容）。

### 本 Feature 新增约束

- zone-map 的 `representative_rect` 必须映射回源图坐标（在 rect 参数指定时相对于源图）
- exposure 的 spot 模式必须同时提供 --x 和 --y，缺一返回 INVALID_PARAMETERS
- Zone label 使用罗马数字字符串（"0"/"I"/"II"/.../"X"），zone 字段用 u8（0-10）
- focus-map 的 `rows * cols` 复用 tile 数量上限（64）
- focus-map 的每个 `cell.region` 必须使用源图坐标
- white-balance 的 channel mean 为 0 时，对应 gain 取 1.0，避免除零和误导性无限增益
- white-balance bias 阈值使用通道差相对均值 5%；低于阈值视为 neutral

### 错误码

| 错误码 | 触发条件 | 触发命令 |
|--------|---------|---------|
| `INVALID_PARAMETERS` | --mode 不是 4 种之一 / spot 缺 --x 或 --y / rows-cols 非法 | exposure / focus-map |
| `INVALID_COORDINATES` | --rect 越界 / spot 的 --x/--y 越界 | 全部 |
| `FILE_NOT_FOUND` | 文件不存在 | 全部 |
| `UNSUPPORTED_FORMAT` | 不支持的图片格式 | 全部 |
| `PIXEL_LIMIT_EXCEEDED` | 超过 100MP | 全部 |

---

## 技术选型

> 引用 project.md 已有选型，无新增依赖。

| 层 | 选择 | 说明 |
|---|------|------|
| 图片加载 | source::load_rgba_source（现有） | 复用 |
| 区域校验 | region::validate_rect（现有） | 复用 |
| 区域遍历 | iterate_region（photo.rs 现有） | 复用 |
| 亮度计算 | luma / luma_u8（photo.rs 现有） | 复用 |
| 区域加载 | load_region（photo.rs 现有） | 复用 |
| 色彩空间转换 | 无（不需要，Zone/EV 都基于 luma） | — |
| 白平衡估计 | 灰世界 RGB 均值 | 复用 iterate_region，无新增依赖 |

---

## 代码映射

```
contract.md ──────────→ crates/core/src/protocol.rs  (所有 Output/Metrics 类型)
                        crates/core/src/photo.rs      (所有 execute_* + 算法)

CLI 命令注册 ─────────→ crates/cli/src/main.rs        (Commands enum + dispatch)
                        crates/cli/src/commands/photo.rs (新增 run_* + ExposureArgs/FocusMapArgs)
                        crates/cli/src/parse.rs        (测光模式解析)

测试 ─────────────────→ crates/core/src/photo.rs       (#[cfg(test)] mod tests)
                        crates/cli/tests/integration_test.rs
                        crates/cli/tests/schema_snapshot_test.rs
```

---

## 编排

### CLI 入口

`crates/cli/src/main.rs` — 命令注册 + dispatch。

### 命令注册

```rust
// main.rs Commands enum 新增:
ZoneMap(commands::photo::RegionArgs),
Exposure(commands::photo::ExposureArgs),
FocusMap(commands::photo::FocusMapArgs),
WhiteBalance(commands::photo::RegionArgs),

// Histogram 已有，改为接受 --rgb flag:
Histogram(commands::photo::HistogramArgs),  // RegionArgs → HistogramArgs（+rgb: bool）
```

### 调用链

```
histogram --rgb:
  main.rs → commands::photo::run_histogram(HistogramArgs)
    → parse_optional_rect_arg
    → core::photo::execute_histogram(input, rect, rgb: bool)
      → load_region → iterate_region + RGB bins
      → CommandResult<HistogramOutput>

zone-map:
  main.rs → commands::photo::run_zone_map(RegionArgs)
    → parse_optional_rect_arg
    → core::photo::execute_zone_map(input, rect)
      → load_region → iterate_region + zone 分桶 + representative_rect
      → CommandResult<ZoneMapOutput>

exposure:
  main.rs → commands::photo::run_exposure(ExposureArgs)
    → parse_optional_rect_arg + parse_metering_mode
    → core::photo::execute_exposure(input, rect, mode, spot_point)
      → load_region → weighted_luma(mode) → EV 计算 → assessment
      → CommandResult<ExposureOutput>

focus-map:
  main.rs → commands::photo::run_focus_map(FocusMapArgs)
    → parse_optional_rect_arg
    → core::photo::execute_focus_map(input, rect, rows, cols)
      → load_region → validate_tile_count/validate_tile_fits
      → measure_sharpness per cell → best_cell + focus_point
      → CommandResult<FocusMapOutput>

white-balance:
  main.rs → commands::photo::run_white_balance(RegionArgs)
    → parse_optional_rect_arg
    → core::photo::execute_white_balance(input, rect)
      → load_region → iterate_region RGB means
      → gray_world_gains + temperature/tint assessment
      → CommandResult<WhiteBalanceOutput>
```

### 测试策略

| 类型 | 覆盖 | 说明 |
|------|------|------|
| 单元测试 | photo.rs 内 | zone_index 映射、weighted_luma 各模式、EV 计算、assessment 边界、focus-map 网格、white-balance bias |
| 集成测试 | cli/tests/ | 摄影计量命令的 CLI 参数解析 + JSON 输出验证 |
| Schema 快照 | cli/tests/ | histogram --rgb、zone-map、exposure、focus-map、white-balance 的 JSON 形状锁定 |
| 向后兼容 | cli/tests/ | histogram 不带 --rgb 的输出与现有 snapshot 完全一致 |

---

## 验收条件追溯

| PRD AC | Contract 对应 | 实现位置 |
|--------|-------------|---------|
| AC-01-1 (RGB 直方图) | HistogramMetrics.rgb: Option<RgbHistogram> | photo.rs execute_histogram |
| AC-01-2 (向后兼容) | rgb=None 时无 rgb 字段 | photo.rs execute_histogram |
| AC-01-3 (rect 区域) | load_region + region | photo.rs |
| AC-01-4 (通道 clipping) | ChannelHistogram.clipping_low/high | photo.rs |
| AC-02-1 (11 zones) | ZoneMapOutput.zones: Vec<ZoneInfo> | photo.rs execute_zone_map |
| AC-02-2 (纯黑) | Zone 0 全量 | photo.rs zone_index + 单元测试 |
| AC-02-3 (纯白) | Zone X 全量 | photo.rs zone_index + 单元测试 |
| AC-02-4 (rect 区域) | load_region + region | photo.rs |
| AC-02-5 (representative_rect) | ZoneInfo.representative_rect | photo.rs |
| AC-03-1~3 (evaluative+assessment) | weighted_luma + EV + assessment | photo.rs execute_exposure |
| AC-03-4 (spot) | spot_point + 单像素 luma | photo.rs + ExposureArgs |
| AC-03-5 (center-weighted) | Gaussian 加权 | photo.rs weighted_luma |
| AC-03-6 (highlight-weighted) | top 10% 均值 | photo.rs weighted_luma |
| AC-03-7 (spot 缺参数) | INVALID_PARAMETERS | parse.rs + commands/photo.rs |
| AC-03-8 (无效模式) | INVALID_PARAMETERS | parse.rs |
| AC-03-9 (rect 区域) | load_region + region | photo.rs |
| AC-03-10 (assessment 边界) | FD5: ±0.5 EV 阈值 | photo.rs + 单元测试 |
| AC-04-1 (focus-map 输出) | FocusMapOutput / FocusCell | photo.rs execute_focus_map |
| AC-04-2 (网格校验) | validate_tile_count / validate_tile_fits | photo.rs execute_focus_map |
| AC-04-3 (rect 区域) | load_region + cell.region 源图坐标 | photo.rs |
| AC-04-4 (余数像素) | FD7 tile remainder 规则 | photo.rs + 单元测试 |
| AC-05-1 (中性灰) | WhiteBalanceMetrics.assessment = neutral | photo.rs execute_white_balance |
| AC-05-2 (偏暖) | temperature_bias = warm + RGB gains | photo.rs |
| AC-05-3 (偏冷) | temperature_bias = cool + RGB gains | photo.rs |
| AC-05-4 (tint) | tint_bias = green/magenta | photo.rs |
| AC-05-5 (rect 区域) | load_region + region | photo.rs |
