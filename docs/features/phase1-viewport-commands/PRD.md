# PRD — Phase 1: 视野控制命令集

> vistools CLI 的 4 个视野核心命令：inspect / overview / tile / viewport，以及第一批视觉仪器中的 sample。全部 JSON 输出 + Agent-safe；生成视野的命令附带坐标映射。

## 版本信息

| 项目 | 值 |
|------|-----|
| 版本 | v1.0 |
| 日期 | 2026-06-01 |
| 作者 | ZeroZ-lab |
| 状态 | 已确认 |

---

## 目标用户

| 角色 | 描述 | 核心诉求 |
|------|------|---------|
| AI Agent | Claude Code / Cursor / Codex 等 Agent | 通过 Bash 调用 CLI 控制视觉输入（inspect→tile→crop→定位问题） |
| 开发者 | 前端/全栈开发者 | 手动检查大截图、切分设计稿、裁剪局部 |

---

## 约束

### 统一坐标系（PD2）

```
原点：左上角 (0, 0)
x：向右递增（像素）
y：向下递增（像素）
rect：{ x, y, width, height }（像素）
percent：{ x, y, w, h }（0.0 - 1.0，相对源图）
anchor：top-left / top / top-right / left / center / right / bottom-left / bottom / bottom-right
```

所有输出包含 `coordinate_mapping` 字段，回答"这个输出在源图的什么位置"。

### Agent-safe（PD3）

- 不覆盖源文件：所有输出命令必须指定输出路径
- 路径 sandbox：拒绝包含 `..` 的路径组件
- tile 限制：最大 rows × cols ≤ 64
- 像素限制：输入图片最大 100MP（10000×10000）
- 错误也返回 JSON（`{ "ok": false, "error": { "code": "...", "message": "..." } }`）

### 稳定错误码（PD4）

| 错误码 | 含义 |
|--------|------|
| `FILE_NOT_FOUND` | 输入文件不存在 |
| `UNSUPPORTED_FORMAT` | 不支持的图片格式 |
| `INVALID_DIMENSIONS` | 图片尺寸为 0 或传入区域宽高为 0 |
| `INVALID_COORDINATES` | 裁剪/视口/sample 坐标超出图片范围 |
| `INVALID_PARAMETERS` | 参数值不合法（如 rows=0） |
| `OUTPUT_WRITE_ERROR` | 输出文件写入失败 |
| `PATH_ESCAPE` | 路径包含 `..` 逃逸 |

---

## 用户故事

### US-01: inspect — 查看图片元数据

**作为** Agent，**我想要** 获取图片的宽高、格式、文件大小等信息，**以便** 决定下一步操作（是否需要裁剪、分几块、用什么策略）。

**验收条件：**

```
Given 一张 6000×4000 的 PNG 文件（24MP）
When 运行 image-viewport inspect screenshot.png --json
Then 返回 JSON 包含：
  ok: true
  source: { width: 6000, height: 4000, format: "png", size_bytes: N }
  suggestion: {
    max_tile_rows: 4,
    max_tile_cols: 3,
    needs_overview: true,
    recommended_next: "overview",
    suggested_max_side: 1568
  }
```

```
Given 一张不存在的文件
When 运行 image-viewport inspect missing.png --json
Then 返回 JSON 包含：
  ok: false
  error: { code: "FILE_NOT_FOUND", message: "..." }
```

```
Given 一张 200×150 的小图片
When 运行 image-viewport inspect small.png --json
Then 返回 JSON 包含：
  suggestion: { needs_overview: false, recommended_next: "direct" }
```

**优先级**：P0

---

### US-02: overview — 生成缩放总览

**作为** Agent，**我想要** 把大图缩放到适合模型查看的尺寸，**以便** 先"扫一眼全局"再决定关注哪个区域。

**验收条件：**

```
Given 一张 6000×4000 的 PNG
When 运行 vistools overview screenshot.png overview.jpg --max-side 1200
Then 生成 overview.jpg（宽 1200px，按比例缩放）
  返回 JSON 包含：
  ok: true
  source: { width: 6000, height: 4000 }
  result: { width: 1200, height: 800, size_bytes: N }
  scale_factor: 0.2
  coordinate_mapping: { scale: 0.2, "source_to_overview": "overview_x = source_x * 0.2" }
```

```
Given max-side 大于原图长边
When 运行 vistools overview small.png out.png --max-side 2000
Then 不放大，直接复制原图，输出 warnings: ["max_side ... copying without scaling"]
```

```
Given 输出路径与输入路径相同
When 运行 vistools overview input.png input.png --max-side 1200
Then 返回错误 ok: false, error: { code: "OUTPUT_SAME_AS_INPUT" }
```

**优先级**：P0

---

### US-03: tile — 网格分块切割

**作为** Agent，**我想要** 把大图切成 rows×cols 的网格，**以便** 逐块检查每个区域。

**验收条件：**

```
Given 一张 6000×4000 的 PNG
When 运行 image-viewport tile screenshot.png --rows 4 --cols 3 --out-dir ./tiles --json
Then 生成 12 个文件：tiles/row-0-col-0.png 到 tiles/row-3-col-2.png
  每个 tile 约 2000×1333
  返回 JSON 包含 tiles 数组，每个 tile 包含：
    { path, row, col, width, height, source_region: { x, y, width, height } }
  source_region 描述该 tile 在源图中的精确位置
```

```
Given 图片不能被整除（如 5000×4000 / 3 cols）
When 运行 tile
Then 最后一个 tile 可能略大（含余数像素），所有 tile 无缝覆盖完整图片
```

```
Given rows × cols > 64
When 运行 image-viewport tile big.png --rows 10 --cols 10 --out-dir ./tiles --json
Then 返回错误 ok: false, error: { code: "INVALID_PARAMETERS", message: "rows*cols exceeds limit of 64" }
```

**优先级**：P0

---

### US-04: viewport — 局部视口裁剪

**作为** Agent，**我想要** 按锚点/百分比/矩形裁剪图片局部，**以便** 放大看细节。

**三种裁剪模式：**

#### viewport anchor — 按语义方位裁剪

```
Given 一张 6000×4000 的 PNG
When 运行 image-viewport viewport anchor screenshot.png right-panel.png --anchor right --width 2000 --height 4000 --json
Then 裁剪右半部分（x=4000, y=0, w=2000, h=4000）
  返回 JSON 包含 crop.region 和 coordinate_mapping
```

#### viewport percent — 按百分比裁剪

```
Given 一张 6000×4000 的 PNG
When 运行 image-viewport viewport percent screenshot.png hero.png --x 0 --y 0.1 --w 1 --h 0.3 --json
Then 裁剪区域 (x=0, y=400, w=6000, h=1200)
  返回 JSON 包含 crop.region 和 coordinate_mapping
```

#### viewport rect — 按像素矩形裁剪

```
Given 一张 6000×4000 的 PNG
When 运行 image-viewport viewport rect screenshot.png detail.png --x 4000 --y 2800 --width 2000 --height 1200 --json
Then 精确裁剪指定区域
  返回 JSON 包含 coordinate_mapping.crop_origin_in_source: [4000, 2800]
```

**通用验收条件：**

```
Given 裁剪区域超出图片边界（如 x=5000, width=2000 在 6000 宽图片上）
When 运行 viewport rect
Then 返回错误 ok: false, error: { code: "INVALID_COORDINATES", message: "crop region exceeds source bounds" }
```

```
Given 裁剪面积为 0（如 width=0）
When 运行 viewport
Then 返回错误 ok: false, error: { code: "INVALID_DIMENSIONS" }
```

```
Given percent 参数超出 0..1（如 w=1.5）
When 运行 viewport percent
Then 返回错误 ok: false, error: { code: "INVALID_PARAMETERS" }
```

```
Given percent 区域越界（如 x=0.8, w=0.3）
When 运行 viewport percent
Then 返回错误 ok: false, error: { code: "INVALID_COORDINATES" }
```

**优先级**：P0

---

### US-05: sample — 点/区域取色

**作为** Agent，**我想要** 读取源图某个点或区域的颜色和透明度，**以便** 检查 UI 颜色、透明遮罩、抗锯齿边缘或设计稿还原。

**两种模式：**

#### point — 单点取色

```
Given 一张 PNG 图片
When 运行 vistools sample screenshot.png --x 120 --y 80
Then 返回 JSON 包含：
  ok: true
  sample: {
    mode: "point",
    point: { x: 120, y: 80 },
    color: { rgba: [R,G,B,A], rgb: [R,G,B], hex: "#rrggbb", alpha: A }
  }
```

#### rect — 区域平均色

```
Given 一张 PNG 图片
When 运行 vistools sample screenshot.png --rect 100,80,40,40
Then 返回 JSON 包含：
  ok: true
  sample: {
    mode: "rect",
    region: { x: 100, y: 80, width: 40, height: 40 },
    average: { rgba: [R,G,B,A], rgb: [R,G,B], hex: "#rrggbb", alpha: A },
    alpha_stats: { min, max, average, transparent_ratio },
    pixel_count: 1600
  }
```

**通用验收条件：**

```
Given 只传入 --x 或只传入 --y
When 运行 sample
Then 返回错误 ok: false, error: { code: "INVALID_PARAMETERS" }
```

```
Given 同时传入 --x/--y 和 --rect
When 运行 sample
Then 返回错误 ok: false, error: { code: "INVALID_PARAMETERS" }
```

```
Given 点或 rect 超出源图边界
When 运行 sample
Then 返回错误 ok: false, error: { code: "INVALID_COORDINATES" }
```

```
Given rect 宽或高为 0
When 运行 sample
Then 返回错误 ok: false, error: { code: "INVALID_DIMENSIONS" }
```

**优先级**：P0（v0.3 第一视觉仪器）

---

## 非功能需求

### 性能（来自 project.md）

| 操作 | 目标 | 测试方法 |
|------|------|---------|
| inspect | < 1ms | 6000×4000 PNG fixture |
| viewport / crop | < 5ms | 同上 |
| tile（单个 tile） | < 5ms | 同上 |
| overview | < 200ms | 同上 |
| sample point | < 5ms | 小图 fixture |
| sample rect | 与区域像素数线性相关 | 透明 PNG fixture + 常规截图 |

### 二进制大小

- Release build（LTO + strip）：≤ 8MB
- 冷启动 `--help`：< 10ms

### 安全

- 不覆盖源文件（PD3）
- 路径 sandbox（拒绝 `..`）
- 输入图片像素限制 100MP
- tile 数量限制 ≤ 64

### 兼容性

- 支持 PNG、JPEG、WebP、TIFF、BMP、GIF 输入输出
- JPEG 输出默认 quality=95
- JSON schema 向后兼容

---

## 范围排除

| 不做 | 理由 | 后续计划 |
|------|------|---------|
| diff / compare | 需要像素比较算法，Phase 1 聚焦视野控制 | Phase 2 |
| concat / blur / pixelate | 非核心视野控制命令 | Phase 2 |
| 通用 resize / rotate | 属于像素处理库能力，不属于视野导航层 | 不做；overview/viewport 覆盖视野需求 |
| login / 远端 AI | 需要服务端基础设施 | Phase 3 |
| analyze / ocr / semantic-diff | 需要远端 AI 模型 | Phase 3 |
| MCP server | CLI-only 架构决策（第4轮） | 不做 |
| SVG / AVIF 输入 | Phase 1 先支持主流格式 | 后续可选 |
| 递归视野探索（tile→viewport→再 tile） | Phase 1 验证基础命令，Agent 可手动串联 | Phase 2 |

---

## 成功指标

| 指标 | 目标值 | 衡量方式 | 时间窗口 |
|------|--------|---------|---------|
| 4 个命令全部可用 | 100% 通过集成测试 | CI | 完成时 |
| JSON 输出 schema 稳定 | 0 breaking change | schema snapshot 测试 | 完成时 |
| Agent 闭环验证 | ≥3 个前端任务成功 | 手动验证 | 2 周 |
| 二进制大小 | ≤ 8MB | release build 测量 | 完成时 |

---

## 验收计划（轻量模式）

**自验收 checklist：**

1. `cargo test` 全部通过
2. `cargo clippy -- -D warnings` 无警告
3. 手动跑 4 个命令，确认 JSON 输出格式正确
4. 确认二进制 ≤ 8MB
5. 用 Claude Code 在一个真实前端任务中串联 inspect → tile → viewport 验证闭环

---

## 依赖与风险

### 依赖

- image-rs 0.25.x 支持 PNG/JPEG/WebP/TIFF 输入输出
- clap 4.x derive 模式支持子命令嵌套（`viewport anchor/percent/rect`）

### 风险

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| image-rs 对某格式解码慢 | 低 | 中 | 测量后决定是否限制输入格式 |
| tile 余数处理复杂 | 低 | 低 | 明确策略：最后一个 tile 包含余数像素 |
| Agent 不主动调用 CLI | 中 | 高 | Phase 1 核心验证目标，失败则调整策略 |
