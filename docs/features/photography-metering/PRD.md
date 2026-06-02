# PRD — 摄影计量 P0（histogram --rgb / zone-map / exposure）

> Agent 可用摄影语言级计量命令回答"曝光对不对"。

## 版本信息

| 项目 | 值 |
|------|-----|
| 版本 | v1.0 |
| 日期 | 2026-06-02 |
| 作者 | ZeroZ-lab |
| 状态 | 已确认 |

---

## 目标用户

| 角色 | 描述 | 核心诉求 |
|------|------|---------|
| Agent（主用户） | 通过 Bash 调用 CLI 的 AI Agent | 用摄影语言评估照片曝光质量，做批量质检 |
| 摄影师（受益人） | 照片交付前的质检需求 | 不手动逐张看直方图，由 Agent 自动筛选问题照片 |

---

## 约束

> 引用 project.md 共享约束：PD3（Agent-safe 只读）、PD4（稳定错误码）、PD7（纯像素数学不加新依赖）、PD8（photo.rs 内扩展）、PD9（histogram 向后兼容）。

### 本 PRD 新增约束

- Zone System 使用线性 11 区映射：亮度 0-255 线性分为 Zone 0-X（Zone 0 = 0-23, Zone I = 24-46, ..., Zone X = 232-255）
- EV 计算公式：`EV = log2(weighted_mean_luma / 118)`，其中 118 是 sRGB 中灰亮度值（Zone V 的中心值）
- 测光模式仅 4 种：evaluative（全局均值）、spot（单点）、center-weighted（高斯加权）、highlight-weighted（仅计 top 10% 亮度像素）
- zone-map 的 `representative_rect` 必须映射回源图坐标

### 错误码

| 错误码 | 含义 | 触发命令 |
|--------|------|---------|
| `FILE_NOT_FOUND` | 输入文件不存在 | 全部 |
| `UNSUPPORTED_FORMAT` | 不支持的图片格式 | 全部 |
| `INVALID_COORDINATES` | rect 越界 | 全部（--rect） |
| `INVALID_PARAMETERS` | 参数格式/值错误 | exposure（--mode 无效、spot 缺 --x/--y） |
| `PIXEL_LIMIT_EXCEEDED` | 超过 100MP 限制 | 全部 |

---

## 用户故事

### US-01: histogram --rgb — 查看 RGB 三通道直方图

**作为** Agent，**我想要** 查看图片的 R/G/B 三通道独立直方图，**以便** 发现单通道溢出（如红通道完全过曝但亮度直方图看起来正常）。

**优先级**：P0

**验收条件（Given-When-Then）：**

#### AC-01-1: RGB 三通道直方图正常输出

```
Given 一张包含红/绿/蓝三色区域的 256x256 PNG
When 运行 vistools histogram test.png --rgb --json
Then 返回 ok=true，data.histogram 包含 luma（现有）+ r/g/b 三个 ChannelHistogram
  每个 ChannelHistogram 包含 bins[256]、mean、p05、p50、p95、clipping_low、clipping_high
```

#### AC-01-2: 向后兼容（不传 --rgb）

```
Given 任意图片
When 运行 vistools histogram test.png --json（不带 --rgb）
Then 输出 JSON 与现有 histogram 完全一致，不包含 r/g/b 字段
```

#### AC-01-3: 指定 rect 区域

```
Given 一张 1000x1000 PNG
When 运行 vistools histogram test.png --rect 100,100,200,200 --rgb --json
Then RGB 直方图仅统计 rect 区域内像素
  data.region = {x:100, y:100, width:200, height:200}
```

#### AC-01-4: 单通道 clipping 检测

```
Given 一张红色通道完全过曝的图片（R≥250，G/B 正常）
When 运行 vistools histogram test.png --rgb --json
Then data.r.clipping_high > 0（红通道高光溢出像素 > 0）
  data.g.clipping_high ≈ 0（绿通道无溢出）
  data.b.clipping_high ≈ 0（蓝通道无溢出）
```

---

### US-02: zone-map — Zone System 影调分区

**作为** Agent，**我想要** 查看图片的 Zone System 影调分布，**以便** 判断暗部/高光细节是否丢失（如"Zone 0-II 占 40%，暗部细节严重丢失"）。

**优先级**：P0

**验收条件（Given-When-Then）：**

#### AC-02-1: 正常 zone-map 输出

```
Given 一张包含从纯黑到纯白渐变的 256x256 PNG
When 运行 vistools zone-map test.png --json
Then 返回 ok=true，data.zones 包含 11 个 ZoneInfo
  每个 ZoneInfo 包含 zone(u8)、pixel_count(u64)、ratio(f64)、representative_rect(Rect)
  所有 zone 的 ratio 之和 ≈ 1.0
```

#### AC-02-2: 纯黑图片

```
Given 一张 64x64 纯黑 PNG（所有像素 RGB=0,0,0）
When 运行 vistools zone-map test.png --json
Then Zone 0 的 pixel_count = 64*64 = 4096，ratio ≈ 1.0
  其余 Zone 的 pixel_count = 0
```

#### AC-02-3: 纯白图片

```
Given 一张 64x64 纯白 PNG（所有像素 RGB=255,255,255）
When 运行 vistools zone-map test.png --json
Then Zone X 的 pixel_count = 4096，ratio ≈ 1.0
  其余 Zone 的 pixel_count = 0
```

#### AC-02-4: 指定 rect 区域

```
Given 一张 1000x1000 渐变 PNG
When 运行 vistools zone-map test.png --rect 0,0,500,500 --json
Then zone 统计仅覆盖 rect 区域，data.region = {x:0, y:0, width:500, height:500}
```

#### AC-02-5: representative_rect 映射

```
Given 一张 1000x1000 PNG
When 运行 vistools zone-map test.png --json
Then 每个 zone 的 representative_rect 坐标在源图范围内（0 ≤ x < 1000, 0 ≤ y < 1000）
  且 representative_rect 确实包含至少一个属于该 zone 的像素
```

---

### US-03: exposure — 曝光评估 + 测光模式

**作为** Agent，**我想要** 获取图片的 EV 估算值和曝光评估，**以便** 自动判断"这张照片整体欠曝 1.3 档"并做 assert 断言。

**优先级**：P0

**验收条件（Given-When-Then）：**

#### AC-03-1: evaluative 模式（默认）

```
Given 一张正确曝光的中灰图片（luma ≈ 118）
When 运行 vistools exposure test.png --json
Then data.metering = "evaluative"
  data.ev ≈ 0.0（偏差 ≤ ±0.3）
  data.assessment = "correct"
  data.mean_luma ≈ 118
```

#### AC-03-2: 过曝图片

```
Given 一张过曝图片（luma ≈ 220）
When 运行 vistools exposure test.png --json
Then data.ev > 1.0
  data.assessment = "over"
```

#### AC-03-3: 欠曝图片

```
Given 一张欠曝图片（luma ≈ 30）
When 运行 vistools exposure test.png --json
Then data.ev < -1.0
  data.assessment = "under"
```

#### AC-03-4: spot 测光模式

```
Given 一张 1000x1000 渐变 PNG
When 运行 vistools exposure test.png --mode spot --x 500 --y 500 --json
Then data.metering = "spot"
  data.spot_point = {x: 500, y: 500}
  data.ev 基于 (500,500) 单像素亮度计算
```

#### AC-03-5: center-weighted 模式

```
Given 一张中心亮四周暗的图片
When 运行 vistools exposure test.png --mode center-weighted --json
Then data.metering = "center_weighted"
  data.ev 偏向中心区域亮度（比 evaluative 更高）
```

#### AC-03-6: highlight-weighted 模式

```
Given 一张含高光区域和暗部区域的图片
When 运行 vistools exposure test.png --mode highlight-weighted --json
Then data.metering = "highlight_weighted"
  data.ev 基于亮度 top 10% 像素的均值计算
```

#### AC-03-7: spot 模式缺 --x/--y

```
Given 任意图片
When 运行 vistools exposure test.png --mode spot --json（不带 --x --y）
Then 返回 ok=false，error.code = "INVALID_PARAMETERS"
  error.message 包含 "spot mode requires --x and --y"
```

#### AC-03-8: 无效测光模式

```
Given 任意图片
When 运行 vistools exposure test.png --mode invalid --json
Then 返回 ok=false，error.code = "INVALID_PARAMETERS"
```

#### AC-03-9: 指定 rect 区域

```
Given 一张 1000x1000 PNG
When 运行 vistools exposure test.png --rect 200,200,400,400 --json
Then 测光仅在 rect 区域内计算
  data.region = {x:200, y:200, width:400, height:400}
```

#### AC-03-10: assessment 边界值

```
Given 任意图片
When 运行 vistools exposure test.png --json
Then:
  ev < -0.5 → assessment = "under"
  -0.5 ≤ ev ≤ 0.5 → assessment = "correct"
  ev > 0.5 → assessment = "over"
```

---

## 非功能需求

### 性能

| 命令 | 目标（6000×4000 图片） | 说明 |
|------|----------------------|------|
| `histogram --rgb` | < 100ms | 单次遍历，3 通道计数 |
| `zone-map` | < 100ms | 单次遍历 + zone 分桶 |
| `exposure` | < 50ms（evaluative/spot）< 200ms（center-weighted/highlight-weighted） | 加权计算略慢 |

### 兼容性

- histogram 不传 `--rgb` 时输出与 v0.2.x 完全一致（PD9）
- 新命令 zone-map / exposure 不影响现有命令

### 输出格式

- 所有命令返回 `CommandResult<T>` 信封（PD1）
- 错误使用稳定 error.code（PD4）
- 测量结果必须映射回源图坐标（zone-map 的 representative_rect、exposure 的 region）

---

## 范围排除

| 不做 | 理由 | 后续计划 |
|------|------|---------|
| sRGB gamma 反转后的 Zone 映射 | 增加复杂度，线性映射和现有 histogram 一致 | 如需精确可在 P1 增加非线性映射 |
| RGB 色彩空间的 Zone System | 摄影师看的是亮度 Zone，不看通道 Zone | — |
| EV 绝对值（基于 EXIF 推算场景亮度） | 需要 EXIF 解析 + 场景反推，超出纯像素范畴 | — |
| 自动曝光建议（"建议 +1.3 EV"） | 需要场景语义（什么是"正确曝光"），纯计量不应带建议 | 可由 Agent 基于数据自行判断 |
| focus-map / white-balance / gamut / noise | P1/P2，等 P0 跑通再评估 | photography-metering idea-brief P1/P2 |
| 多图批量处理 | 编排由 Agent 负责，vistools 只做单图原子命令 | v1 命令面原则 |
| 自然语言报告 | 由 LLM 基于结构化结果完成 | v1 排除 |

---

## 成功指标

| 指标 | 目标值 | 衡量方式 | 时间窗口 |
|------|--------|---------|---------|
| Agent 曝光判断准确率 | ≥ 90%（与 Lightroom 判断一致） | 50 张真实照片对比 | 实现后 1 周 |
| histogram --rgb 向后兼容 | 0 个现有测试回归 | cargo test 全量通过 | 实现后立即 |
| 性能达标 | 3 个命令均满足性能目标 | benchmark | 实现后立即 |

---

## 验收计划

**验收人**：ZeroZ-lab

**验收流程**：
1. `cargo test` 全量通过 + `cargo clippy` 0 warnings
2. 用 fixtures/e2e/ 真实图片验证 3 个命令输出
3. 对比 histogram 不带 --rgb 的输出与修改前完全一致（schema snapshot 测试）
4. 用真实过曝/欠曝照片验证 exposure 的 assessment 判断
5. schema snapshot 测试覆盖 3 个新命令的 JSON 输出形状

**回退方案**：验收不通过 → 回退到当前 main，问题记录到 changelog。

---

## 依赖与风险

### 依赖

- 现有 `load_region` / `iterate_region` 基础设施（photo.rs）
- 现有 `RegionArgs` CLI 参数模式（cli/commands/photo.rs）
- 现有 `luma` / `luma_u8` 辅助函数（photo.rs）

### 风险

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| EV 线性公式对高对比度场景不准确 | 中 | 中 | assessment 用宽阈值（±0.5），不做精确 EV 断言；精确评估留给 Agent |
| center-weighted 高斯权重的 σ 选择影响结果 | 低 | 低 | σ 取图片短边的 1/3（常见相机默认值），写入 PD 共享决策 |
| histogram --rgb 增加 3×256 bins 导致输出 JSON 过大 | 低 | 低 | 256 bins 已是现有模式，Agent 解析无压力 |
