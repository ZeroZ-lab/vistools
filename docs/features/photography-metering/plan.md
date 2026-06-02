# Plan — 摄影计量 P0

> 3 个命令（histogram --rgb / zone-map / exposure）的垂直切片 + 验证收尾

## 版本信息

| 项目 | 值 |
|------|-----|
| 版本 | v1.0 |
| 日期 | 2026-06-02 |
| 来源 | contract.md（FD1-FD6）+ PRD.md（US-01~03, AC-01~03） |

---

## 依赖图

```
Task-01 (histogram --rgb — 修改现有命令，向后兼容风险最高)
  ↓
Task-02 (zone-map) ← 与 Task-03 并行
Task-03 (exposure) ← 与 Task-02 并行
  ↓
Task-04 (schema snapshot + 集成验证)
```

---

## 并行矩阵

| 任务 | 可并行 | 原因 |
|------|--------|------|
| Task-01 | 否 | 修改现有 histogram + main.rs 注册模式，后续任务依赖此模式 |
| Task-02 | 是（与 Task-03） | 新命令，无共享 execute 函数 |
| Task-03 | 是（与 Task-02） | 新命令，无共享 execute 函数 |
| Task-04 | 否 | 依赖 Task-01~03 全部完成 |

**关键路径**：Task-01 → Task-02 → Task-04（或 Task-01 → Task-03 → Task-04）

**风险排序理由**：Task-01 修改现有命令，向后兼容风险最高，必须先做先验证。

---

## 任务清单

### Task-01: histogram --rgb 增强

**目标**：在现有 histogram 命令基础上增加 `--rgb` flag，输出 R/G/B 三通道直方图，不破坏现有输出。
**PRD 来源**：US-01（AC-01-1, AC-01-2, AC-01-3, AC-01-4）
**依赖**：无
**文件**：
- crates/core/src/protocol.rs（新增 RgbHistogram + ChannelHistogram，修改 HistogramMetrics）
- crates/core/src/photo.rs（修改 execute_histogram 接受 rgb 参数）
- crates/cli/src/commands/photo.rs（新增 HistogramArgs + 修改 run_histogram）
- crates/cli/src/main.rs（Histogram 命令改用 HistogramArgs）
- crates/core/src/photo.rs tests（新增 RGB 测试）

**步骤**：
1. protocol.rs：新增 `ChannelHistogram`（bins/mean/p05/p50/p95/clipping_low/clipping_high）+ `RgbHistogram`
2. protocol.rs：`HistogramMetrics` 新增 `rgb: Option<RgbHistogram>`，加 `#[serde(skip_serializing_if = "Option::is_none")]`
3. photo.rs：修改 `execute_histogram` 签名加 `rgb: bool`；当 rgb=true 时额外统计 R/G/B 三通道
4. cli/commands/photo.rs：新增 `HistogramArgs`（input + rect + rgb: bool），修改 `run_histogram` 传 rgb 参数
5. main.rs：`Commands::Histogram` 改用 `HistogramArgs`
6. photo.rs tests：新增 `rgb_histogram_reports_three_channels` + `rgb_backward_compatible_when_off`
7. `cargo test` + `cargo clippy` 验证

**验证标准**：
- `vistools histogram test.png --json` 输出与修改前完全一致（无 rgb 字段）
- `vistools histogram test.png --rgb --json` 输出包含 r/g/b 三通道数据
- 现有 histogram 单元测试 + 集成测试 + schema snapshot 全部通过
- 新增单元测试：RGB 三通道 + 不传 --rgb 时无 rgb 字段

---

### Task-02: zone-map 命令

**目标**：新增 zone-map 命令，输出 Zone System 11 区分布 + 每区代表区域坐标。
**PRD 来源**：US-02（AC-02-1, AC-02-2, AC-02-3, AC-02-4, AC-02-5）
**依赖**：Task-01（main.rs 注册模式已建立）
**文件**：
- crates/core/src/protocol.rs（新增 ZoneMapOutput + ZoneInfo）
- crates/core/src/photo.rs（新增 execute_zone_map + zone_index）
- crates/cli/src/commands/photo.rs（新增 run_zone_map，复用 RegionArgs）
- crates/cli/src/main.rs（注册 ZoneMap 命令）
- crates/core/src/photo.rs tests（新增 zone-map 测试）

**步骤**：
1. protocol.rs：新增 `ZoneInfo`（zone/label/luma_range/pixel_count/ratio/representative_rect）+ `ZoneMapOutput`（source/region/zones）
2. photo.rs：新增 `zone_index(luma: u8) -> u8`，实现线性映射 `min(10, luma * 11 / 256)`
3. photo.rs：新增 `execute_zone_map`，遍历像素分桶到 11 个 zone，每个 zone 找第一个匹配像素作为 representative_rect
4. cli/commands/photo.rs：新增 `run_zone_map`，复用 `RegionArgs` + `run_region_metric` 模式
5. main.rs：注册 `Commands::ZoneMap(RegionArgs)` + dispatch 到 `run_zone_map`
6. photo.rs tests：新增 `zone_map_black_image`（Zone 0 全量）+ `zone_map_white_image`（Zone X 全量）+ `zone_map_gradient`（11 区均有分布）+ `zone_map_rect_region`
7. `cargo test` + `cargo clippy` 验证

**验证标准**：
- 纯黑图 → Zone 0 ratio ≈ 1.0，其余 0
- 纯白图 → Zone X ratio ≈ 1.0，其余 0
- 渐变图 → 11 个 zone 都有像素，ratio 之和 ≈ 1.0
- representative_rect 坐标在源图范围内
- 指定 --rect 时统计仅覆盖 rect 区域

---

### Task-03: exposure 命令

**目标**：新增 exposure 命令，支持 4 种测光模式，输出 EV + assessment（under/correct/over）。
**PRD 来源**：US-03（AC-03-1 ~ AC-03-10）
**依赖**：Task-01（main.rs 注册模式已建立）
**文件**：
- crates/core/src/protocol.rs（新增 ExposureOutput）
- crates/core/src/photo.rs（新增 execute_exposure + weighted_luma + 4 种加权函数）
- crates/cli/src/commands/photo.rs（新增 ExposureArgs + run_exposure）
- crates/cli/src/parse.rs（新增测光模式解析）
- crates/cli/src/main.rs（注册 Exposure 命令）
- crates/core/src/photo.rs tests（新增 exposure 测试）

**步骤**：
1. protocol.rs：新增 `ExposureOutput`（source/region/metering/ev/assessment/mean_luma/spot_point）
2. photo.rs：新增 4 个加权函数：
   - `luma_evaluative`：全部像素等权均值
   - `luma_spot`：单像素 luma at (x, y)
   - `luma_center_weighted`：高斯加权 σ = min(w,h)/3
   - `luma_highlight_weighted`：top 10% luma 像素均值
3. photo.rs：新增 `execute_exposure(input, rect, mode, spot_point)`，计算加权均值 → EV = log2(luma/118) → assessment（FD3 + FD5）
4. parse.rs：新增 `parse_metering_mode(value: &str) -> Result<MeteringMode, String>`
5. cli/commands/photo.rs：新增 `ExposureArgs`（input/rect/mode/x/y）+ `run_exposure`，含参数校验（spot 必须有 --x/--y）
6. main.rs：注册 `Commands::Exposure(ExposureArgs)` + dispatch
7. photo.rs tests：新增 `exposure_correct_image_ev_near_zero` + `exposure_over_image` + `exposure_under_image` + `exposure_spot_mode` + `exposure_spot_missing_xy` + `exposure_invalid_mode` + `exposure_assessment_boundaries`
8. `cargo test` + `cargo clippy` 验证

**验证标准**：
- 中灰图（luma ≈ 118）→ ev ≈ 0.0, assessment = "correct"
- 过曝图（luma ≈ 220）→ ev > 1.0, assessment = "over"
- 欠曝图（luma ≈ 30）→ ev < -1.0, assessment = "under"
- spot 模式缺 --x/--y → error.code = "INVALID_PARAMETERS"
- 无效 mode → error.code = "INVALID_PARAMETERS"
- assessment 边界值：ev = -0.5 → "correct"，ev = 0.5 → "correct"，ev = -0.6 → "under"

---

### Task-04: Schema snapshot + 集成验证

**目标**：为新命令添加 schema snapshot 测试，验证向后兼容，全量回归。
**PRD 来源**：US-01~03 全部 AC
**依赖**：Task-01 + Task-02 + Task-03
**文件**：
- crates/cli/tests/schema_snapshot_test.rs（新增 3 个 snapshot）
- crates/cli/tests/integration_test.rs（新增 CLI 集成测试）

**步骤**：
1. schema_snapshot_test.rs：新增 `histogram_rgb_schema` snapshot（带 --rgb）
2. schema_snapshot_test.rs：验证 `histogram_schema`（不带 --rgb，现有 snapshot）未变化
3. schema_snapshot_test.rs：新增 `zone_map_schema` snapshot
4. schema_snapshot_test.rs：新增 `exposure_schema` snapshot（evaluative 模式）
5. integration_test.rs：新增 zone-map + exposure 的 CLI 集成测试（参数解析 + JSON 输出 + exit code）
6. `cargo test` 全量 + `cargo clippy` + `cargo fmt --check`
7. 用 fixtures/e2e/ 真实图片手动验证输出合理性

**验证标准**：
- 全量测试通过（现有 + 新增）
- clippy 0 warnings
- fmt clean
- histogram 不带 --rgb 的 schema snapshot 与现有完全一致
- 新增 3 个 schema snapshot 锁定 JSON 形状

---

## 检查点

| 检查点 | 时机 | 验收标准 | 回退方案 |
|--------|------|---------|---------|
| CP-1 | Task-01 完成后 | histogram --rgb 输出正确 + 不带 --rgb 时向后兼容 + 现有测试全绿 | 回退 histogram 修改，RGB 功能放独立命令 |
| CP-2 | Task-04 完成后 | 3 个命令全量测试通过 + schema snapshot 锁定 + clippy 0 warnings | 回退最后一个有问题的 Task |

---

## 执行顺序

```
第 1 步：Task-01（histogram --rgb — 修改现有命令，风险最高先验证）
  → CP-1 检查
第 2 步：Task-02 + Task-03（zone-map / exposure — 两个新命令可并行）
第 3 步：Task-04（schema snapshot + 集成验证）
  → CP-2 检查
```

**预估时间**：
- Task-01: ~1h（修改现有命令，需要仔细验证兼容性）
- Task-02: ~1h（新命令，算法简单）
- Task-03: ~1.5h（4 种测光模式，加权函数较多）
- Task-04: ~0.5h（snapshot + 集成测试）
- **总计：~4h**
