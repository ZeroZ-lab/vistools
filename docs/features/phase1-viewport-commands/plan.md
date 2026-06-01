# Plan — Phase 1: 视野控制命令集

> 任务分解 + 依赖图 + 执行顺序

## 版本信息

| 项目 | 值 |
|------|-----|
| 版本 | v1.0 |
| 日期 | 2026-06-01 |
| 来源 | contract.md + PRD.md |

---

## 依赖图

```
Task-01 (Cargo workspace + types)
  ↓
Task-02 (guard + coord)
  ↓
┌──────────┬──────────┐
↓          ↓          ↓
Task-03   Task-04   Task-05
inspect   overview   tile
(P0)      (P0)      (P0)
  ↓          ↓          ↓
  └──────────┼──────────┘
             ↓
         Task-06
         viewport (P0)
             ↓
      ┌──────┴──────┐
      ↓             ↓
   Task-07       Task-08
   resize (P1)   rotate (P1)
      └──────┬──────┘
             ↓
         Task-09
         CLI main.rs
         (集成 + 全量测试)
```

---

## 并行矩阵

| 任务 | 可并行 | 原因 |
|------|--------|------|
| Task-01 | 否 | 基础设施，所有任务依赖 |
| Task-02 | 否 | guard/coord 被所有命令依赖 |
| Task-03 | 是（与 04/05） | inspect 不写输出文件，与 overview/tile 无共享 |
| Task-04 | 是（与 03/05） | overview 独立写 overview.rs |
| Task-05 | 是（与 03/04） | tile 独立写 tile.rs |
| Task-06 | 否 | viewport 是核心命令，需验证 03-05 模式 |
| Task-07 | 是（与 08） | resize 独立 |
| Task-08 | 是（与 07） | rotate 独立 |
| Task-09 | 否 | 集成所有模块 |

**关键路径**：Task-01 → Task-02 → Task-03 → Task-06 → Task-07 → Task-09

---

## 任务清单

### Task-01: Cargo workspace + 核心类型

**目标**：搭建 Rust workspace 骨架，定义所有共享类型（types.rs）
**依赖**：无
**文件**：
- `Cargo.toml`（workspace root）
- `crates/core/Cargo.toml`
- `crates/core/src/lib.rs`
- `crates/core/src/types.rs`
- `crates/cli/Cargo.toml`
- `crates/cli/src/main.rs`（空壳，能编译）
- `fixtures/`（测试图片）

**步骤**：
1. 创建 Cargo workspace（root Cargo.toml + crates/core + crates/cli）
2. 在 core/src/types.rs 定义所有共享类型：Point、Rect、Percent、Anchor、Size、CommandResult<T>、ErrorInfo、CoordinateMapping、SourceInfo
3. 在 core/src/lib.rs 导出 types
4. cli crate 添加对 core 的依赖
5. cli/src/main.rs 写空壳（能 `cargo build` 通过）
6. 生成测试 fixture 图片（64x64.png、256x256.png、1000x1000.png）

**验证标准**：
- `cargo build` 成功
- `cargo test` 成功（无测试但能跑）
- `cargo clippy -- -D warnings` 无警告
- types 中所有类型实现 Debug + Clone + serde::Serialize

---

### Task-02: guard + coord 模块

**目标**：实现 Agent-safe 校验（guard）和坐标计算（coord）
**依赖**：Task-01
**文件**：
- `crates/core/src/guard.rs`
- `crates/core/src/coord.rs`
- `crates/core/src/lib.rs`（更新导出）
- `tests/guard_test.rs`
- `tests/coord_test.rs`

**步骤**：
1. guard.rs：实现 `validate_input_path`、`validate_output_path`、`validate_dimensions`（100MP）、`validate_tile_count`（≤64）、`validate_different_paths`（输出≠输入）
2. guard.rs：每个函数返回 Result，错误映射到 ErrorCode
3. coord.rs：实现 `anchor_to_rect(anchor, viewport_w, viewport_h, source_size) → Rect`
4. coord.rs：实现 `percent_to_rect(percent, source_size) → Rect`
5. coord.rs：实现 `make_mapping(source_rect, source_size, result_size) → CoordinateMapping`
6. 写 guard_test.rs：路径 sandbox、像素限制、tile 限制
7. 写 coord_test.rs：anchor 九宫格计算、percent 转像素、映射公式生成

**验证标准**：
- `cargo test` 全部通过
- guard 拒绝 `../etc/passwd`、10001x10001 图片、10x10 tiles
- coord: anchor right 在 6000x4000 图上 → x=4000, y=0
- coord: percent (0.5, 0.5, 0.5, 0.5) 在 1000x1000 图上 → (500, 500, 500, 500)

---

### Task-03: inspect 命令

**目标**：实现 inspect 命令——读图片元数据 + 策略建议（P0，最高优先）
**依赖**：Task-02
**文件**：
- `crates/core/src/inspect.rs`
- `crates/core/src/lib.rs`（更新导出）
- `tests/inspect_test.rs`

**步骤**：
1. 实现 `inspect::execute(input_path) -> CommandResult<InspectOutput>`
2. 用 `image::image_dimensions()` 读宽高（不加载全图）
3. 读文件大小（`std::fs::metadata`）
4. 推断格式（基于文件扩展名或 image-rs decoder）
5. 计算 suggestion（needs_overview: 长边 > 1568, max_tile_rows/cols）
6. 写测试：正常图片、不存在文件、超像素图片、小图片

**验证标准**：
- inspect 6000x4000 PNG → `{ width: 6000, height: 4000, suggestion: { needs_overview: true } }`
- inspect 不存在文件 → `{ ok: false, error: { code: "FILE_NOT_FOUND" } }`
- inspect 200x150 小图 → `{ suggestion: { needs_overview: false } }`
- 耗时 < 1ms（不加载全图像素）

---

### Task-04: overview 命令

**目标**：实现 overview 命令——缩放生成总览图（P0）
**依赖**：Task-02
**文件**：
- `crates/core/src/overview.rs`
- `crates/core/src/lib.rs`（更新导出）
- `tests/overview_test.rs`

**步骤**：
1. 实现 `overview::execute(input, output, max_width) -> CommandResult<OverviewOutput>`
2. guard 校验输入/输出路径 + 像素限制
3. 加载图片 → 按比例缩放到 max_width → 保存到 output
4. 如果 max_width > 源宽，直接复制，加 warning
5. 计算 scale_factor + CoordinateMapping
6. 写测试：正常缩放、max_width > 源图、输出=输入

**验证标准**：
- overview 6000x4000 → max-width 1200 → 输出 1200x800 + scale_factor 0.2
- max-width 2000 在 1000px 图上 → 不放大，warning
- 输出=输入路径 → OUTPUT_SAME_AS_INPUT 错误
- coordinate_mapping 含公式 `overview_x = source_x * 0.2`

---

### Task-05: tile 命令

**目标**：实现 tile 命令——网格分块切割（P0）
**依赖**：Task-02
**文件**：
- `crates/core/src/tile.rs`
- `crates/core/src/lib.rs`（更新导出）
- `tests/tile_test.rs`

**步骤**：
1. 实现 `tile::execute(input, rows, cols, out_dir) -> CommandResult<TileOutput>`
2. guard 校验 tile_count ≤ 64 + 像素限制 + 路径
3. 创建 out_dir（如不存在）
4. 计算每个 tile 的源区域（含余数策略：最后一个 tile 含余数像素）
5. 逐块 crop + 保存，命名 `row-N-col-M.<ext>`
6. 生成 TileInfo 数组（path/row/col/size/source_region）
7. 写测试：4x3 切割、余数处理（5000/3 cols）、超限 tiles

**验证标准**：
- 6000x4000 / 4x3 → 12 个 tile，每个约 2000x1333
- 5000x4000 / 3 cols → 最后 col 宽 1668（含余数）
- 10x10 tiles → INVALID_PARAMETERS 错误
- 所有 tile source_region 无缝覆盖完整源图

---

### Task-06: viewport 命令（anchor/percent/rect）

**目标**：实现 viewport 三种裁剪模式（P0，核心命令）
**依赖**：Task-03/04/05（验证 core 模式后实现）
**文件**：
- `crates/core/src/viewport.rs`
- `crates/core/src/lib.rs`（更新导出）
- `tests/viewport_test.rs`

**步骤**：
1. 实现 `viewport::execute(input, output, rect, mode) -> CommandResult<ViewportOutput>`
2. guard 校验路径 + 坐标不越界 + 裁剪面积 > 0
3. 对 anchor 模式：调 `coord::anchor_to_rect` 计算像素区域
4. 对 percent 模式：调 `coord::percent_to_rect` 计算像素区域
5. 对 rect 模式：直接使用像素值
6. crop + 保存 + 生成 CoordinateMapping
7. 写测试：三种模式各正常+越界+零面积

**验证标准**：
- anchor right, w=2000, h=4000 在 6000x4000 → crop (4000, 0, 2000, 4000)
- percent (0, 0.1, 1, 0.3) 在 6000x4000 → crop (0, 400, 6000, 1200)
- rect (4000, 2800, 2000, 1200) → 精确裁剪
- 越界 → INVALID_COORDINATES
- width=0 → INVALID_DIMENSIONS
- coordinate_mapping.crop_origin_in_source 正确

---

### Task-07: resize 命令

**目标**：实现 resize 命令（P1）
**依赖**：Task-06
**文件**：
- `crates/core/src/resize.rs`
- `crates/core/src/lib.rs`（更新导出）
- `tests/resize_test.rs`

**步骤**：
1. 实现 `resize::execute(input, output, width, height?) -> CommandResult<ResizeOutput>`
2. width only → 等比例缩放；width + height → 强制尺寸
3. 计算 scale_factor + CoordinateMapping
4. 写测试：等比例、强制、缺 width

**验证标准**：
- 6000x4000 --width 1568 → 1568x1045
- --width 800 --height 600 → 800x600
- 缺 --width → INVALID_PARAMETERS
- coordinate_mapping 含 scale_factor

---

### Task-08: rotate 命令

**目标**：实现 rotate 命令（P1）
**依赖**：Task-06
**文件**：
- `crates/core/src/rotate.rs`
- `crates/core/src/lib.rs`（更新导出）
- `tests/rotate_test.rs`

**步骤**：
1. 实现 `rotate::execute(input, output, degrees) -> CommandResult<RotateOutput>`
2. 校验 degrees ∈ {90, 180, 270}；degrees=0 → 复制 + warning
3. 调 image-rs rotate90/180/270
4. 生成旋转后 CoordinateMapping（含旋转公式）
5. 写测试：90°/180°/270°/0°/45°

**验证标准**：
- 6000x4000 --degrees 90 → 4000x6000
- --degrees 0 → 复制原图 + warning
- --degrees 45 → INVALID_PARAMETERS
- coordinate_mapping 含旋转坐标公式

---

### Task-09: CLI 集成 + 全量测试

**目标**：实现 CLI 入口（clap），集成所有命令，跑全量集成测试
**依赖**：Task-07 + Task-08
**文件**：
- `crates/cli/src/main.rs`（完整实现）
- `tests/integration_test.rs`（全量 E2E）

**步骤**：
1. main.rs：用 clap derive 定义 CLI 结构（命令 + 子命令 + 参数）
2. 每个命令：解析参数 → guard 校验 → 调 core → JSON 输出 stdout
3. viewport 子命令：`image-viewport viewport {anchor|percent|rect} ...`
4. `--json` 标志（默认 true，`--quiet` 只输出错误码）
5. 写 integration_test.rs：用 assert_cmd 测试每个命令的 CLI 调用 + JSON 输出
6. 验证二进制大小 ≤ 8MB（`cargo build --release` 后检查）

**验证标准**：
- `image-viewport inspect fixtures/256x256.png --json` 输出合法 JSON
- `image-viewport tile fixtures/1000x1000.png --rows 2 --cols 2 --out-dir /tmp/tiles --json` 生成 4 个文件
- `image-viewport viewport anchor fixtures/1000x1000.png /tmp/crop.png --anchor right --width 500 --height 1000 --json` 裁剪正确
- 所有命令错误情况输出 `{"ok": false, "error": {"code": "..."}}`
- `cargo build --release` 产物 ≤ 8MB

---

## 检查点

| 检查点 | 时机 | 验收标准 | 回退方案 |
|--------|------|---------|---------|
| CP-1 | Task-02 完成后 | guard/coord 单元测试全过，anchor 九宫格计算正确 | 重写 coord 算法 |
| CP-2 | Task-06 完成后 | P0 四命令（inspect/overview/tile/viewport）可用 | 修复具体命令 |
| CP-3 | Task-09 完成后 | CLI 集成测试全过，二进制 ≤ 8MB | 检查依赖树 |

---

## 执行顺序

```
第 1 步：Task-01（串行，~30min）
第 2 步：Task-02（串行，~1h）
第 3 步：Task-03 + Task-04 + Task-05（并行，~2h）
第 4 步：Task-06（串行，~1.5h）
第 5 步：Task-07 + Task-08（并行，~1h）
第 6 步：Task-09（串行，~1.5h）

预估总时间：~7.5h（不含 review）
关键路径：Task-01 → 02 → 03 → 06 → 07 → 09
```

---

## 规划决策

### PL-1: 先 P0 后 P1

**选择**：Task-03/04/05/06（P0）先于 Task-07/08（P1）
**理由**：P0 四命令覆盖核心视野控制闭环。如果 P0 验证失败（Agent 不调用），P1 不值得做。
**拒绝**：按模块字母序（无优先级意识）

### PL-2: Task-03/04/05 并行

**选择**：inspect/overview/tile 可并行开发
**理由**：三者无共享文件（各自独立 .rs），都只依赖 types + guard + coord。并行节省 ~2h。
**拒绝**：串行（浪费时间）

### PL-3: CLI 集成放最后

**选择**：Task-09 在所有命令逻辑完成后做
**理由**：CLI 是薄层（参数解析 + JSON 输出），不涉及业务逻辑。先验证 core 库正确，最后接 CLI。
**拒绝**：边写命令边写 CLI（CLI 变动频繁，浪费时间）
