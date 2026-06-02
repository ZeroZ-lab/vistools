# vistools — AI 行为指令

> 从 project.md 投影生成，不要手写。

## 角色

你是 vistools 的开发搭档。这是一个给 AI Agent 用的本地图片视野控制 CLI 工具——JSON-first、坐标映射、Agent-safe。

## 技术栈（来自 project.md）

- 语言: Rust 1.94+
- 图片处理: image-rs 0.25.x
- CLI: clap 4.x
- EXIF: kamadak-exif
- 分发: cargo-dist
- 测试: Rust 内置 + assert_cmd

## 命令

```bash
# 开发
cargo run -- inspect test.png --json

# 构建（release）
cargo build --release

# 测试（全量）
cargo test

# 测试（单文件）
cargo test --test inspect_test

# 类型检查 / lint
cargo clippy -- -D warnings
cargo fmt --check
```

## 项目结构

```
vistools/
├── Cargo.toml
├── Cargo.lock
├── .gitignore
├── CLAUDE.md
├── AGENTS.md
├── docs/
│   ├── project.md
│   ├── idea-brief.md
│   ├── timeline.md
│   ├── status.md
│   └── features/
│       ├── phase1-viewport-commands/
│       ├── v1-agent-command-surface/
│       └── photography-metering/
├── crates/
│   ├── core/src/
│   │   ├── lib.rs
│   │   ├── constants.rs    # 共享常量
│   │   ├── error.rs        # 稳定错误码 + ErrorInfo
│   │   ├── geom.rs         # Point/Rect/Percent/Anchor/Size
│   │   ├── protocol.rs     # CommandResult<T> + 所有 Output/Metrics 类型
│   │   ├── source.rs       # 图片加载、metadata、format 推断、像素限制
│   │   ├── region.rs       # anchor/percent/rect 解析、clamp、mapping
│   │   ├── coord.rs        # 坐标计算 + 映射
│   │   ├── guard.rs        # Agent-safe 校验
│   │   ├── util.rs         # 共享 save_image
│   │   ├── inspect.rs
│   │   ├── overview.rs
│   │   ├── tile.rs
│   │   ├── viewport.rs
│   │   ├── sample.rs       # 取色器
│   │   ├── photo.rs        # 摄影计量（sharpness/histogram/clipping/contrast/color-cast/zone-map/exposure/focus-map/white-balance）
│   │   └── test_support.rs # 测试 fixture 路径工具
│   └── cli/
│       ├── src/
│       │   ├── main.rs         # CLI 入口（命令注册 + dispatch）
│       │   ├── parse.rs        # 共享 CLI 参数解析
│       │   └── commands/       # 每个命令的 clap 参数 + 调用适配
│       │       ├── mod.rs
│       │       ├── inspect.rs
│       │       ├── overview.rs
│       │       ├── tile.rs
│       │       ├── viewport.rs
│       │       ├── sample.rs
│       │       └── photo.rs
│       └── tests/
│           ├── integration_test.rs
│           └── schema_snapshot_test.rs
├── fixtures/
│   ├── 64x64.png
│   ├── 256x256.png
│   ├── 1000x1000.png
│   └── e2e/
│       ├── landscape_large.jpg
│       ├── portrait_tall.jpg
│       ├── panorama_wide.jpg
│       ├── urban_square.jpg
│       ├── screenshot_like.jpg
│       ├── nature_small.jpg
│       └── nature_small.png
└── docs/
```

## 工作流

- 新命令 → 先写 protocol.rs 类型，再写 photo.rs 逻辑，再写 CLI 命令适配
- 加功能 → 参考已有命令模式（clap derive + core 调用 + JSON 输出）
- 改决策 → 更新 project.md PD# 编号，重新审视受影响命令
- 每个关键逻辑分支注释决策编号（FD# feature 级 / PD# 项目级）
- 测试失败 → 读 project.md 共享决策找分歧，修代码对齐

## AI 执行纪律

- 改动前先确认目标、边界、假设和需要同步的契约文件。
- 优先做满足当前目标的最小变更，不引入未要求的抽象、配置或兼容层。
- 只编辑与目标直接相关的文件；发现无关问题只记录，不顺手修改。
- 每次代码变更后，运行 `cargo test` + `cargo clippy` 验证。

## Skill 调用深度

- L0 lens：只分析、判断或 review 一个点，不改文件。
- L1 patch：局部修改代码或文档，必须执行可用验证。
- L2 stage：完整阶段执行，必须产出或更新阶段文档和历史记录。
- 用户未显式点名阶段时，默认选择最小相关 skill 做 L0/L1 轻量调用。
- 用户显式点名阶段或 skill 时，默认 L2 阶段调用。

## 代码标准

- 所有命令必须输出 JSON（`--json` 标志或默认）
- 坐标类型用 newtype（`Point`、`Rect`、`Percent`、`Anchor`），不用裸 tuple
- 错误返回稳定 `error.code`，Agent 可 pattern-match
- 不覆盖源文件，所有输出必须指定路径
- cargo clippy 0 warnings，cargo fmt 格式化

## 设计约束

- 纯 CLI 工具，无前端、无 Web UI
- 输出是给 Agent 读的 JSON，不是给人读的文本
- 坐标映射是核心：每个操作都要回答"这个输出在源图的什么位置"
- 二进制 ≤8MB（Phase 1），无 C 依赖
- 摄影计量全部只读，纯像素数学，不加新依赖（PD7）
- 摄影计量命令全部在 photo.rs 内扩展，不加新模块（PD8）
- histogram --rgb 向后兼容，不传 --rgb 时输出不变（PD9）

## 边界

### Always
- 新命令必须有 `--json` 输出 + 坐标映射
- `cargo test` 通过再提交
- 读 project.md 确认共享决策
- 错误也返回结构化 JSON
- 摄影计量命令复用 `load_region` / `iterate_region` 基础设施

### Ask First
- 添加新依赖
- 修改 JSON 输出 schema（影响 Agent 兼容性）
- 修改 CLI 参数（影响用户脚本）
- 添加 feature flag

### Never
- 提交 API key 或 token
- 删除失败的测试（应该修代码）
- 用 unwrap()（用 `?` 或显式错误处理）
- 覆盖源文件
- 引入 C 依赖（破坏单二进制分发）

## 历史维护（自动，每次文档变更后执行）

- 改完文档 → 追加 docs/features/<feature>/changelog.md（触发 + 产出 + 决策）
- 完成阶段 → 追加 docs/timeline.md（一条 = 一次发布）
- 每次开发前 → 读 timeline.md 了解上下文
- timeline.md 超 100 行 → 旧记录归档到 timeline/

## 文档引用

| 文件 | 用途 |
|------|------|
| docs/project.md | 技术决策 + 共享约束 |
| docs/idea-brief.md | 探索阶段分析（竞品/场景/评分） |
| docs/timeline.md | 项目演进时间线 |
| docs/features/<feature>/contract.md | 功能合约 |
| docs/features/<feature>/changelog.md | 功能变更历史 |
