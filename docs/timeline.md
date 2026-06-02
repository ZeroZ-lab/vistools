### 2026-06-01 — Agent Image Viewport CLI 探索
- 产出：idea-brief.md（3 个主方向 + 8 个已评估方向，2 轮探索，推荐：AI Coding Screenshot Inspector + 通用视野 CLI 基底）
- 核心结论：方向成立。产品定位为"Agent 的视觉视野控制层"，非图片处理工具。MVP 8 个命令，2 周内可验证。
- 第2轮发现：
  - 直接竞品 imgctl (agent-rt/imgctl) 已覆盖 crop/slice/diff，但缺少 viewport 导航抽象
  - 30+ 生态工具证明需求真实（Peekaboo 4.6k stars、gsd-browser 235 stars）
  - Claude Code 20+ issues 验证 AI Coding 截图检查是强痛点
  - 真正 gap：没有工具做"可导航的视觉视野"——递归探索 + 坐标系统
  - 策略调整：从"图片处理"转向"视野导航"，做比 imgctl 更高层次的抽象
- 第3轮发现：
  - imgctl 深挖：16命令，v0.1.0，0 stars，macOS only，无 MCP，1人项目。可在 1-2 周内添加基础 viewport 但深层差异需数月
  - 痛点升级：80+ Claude Code issues（非20+），10+ session-bricking bugs，Agent "声称完成但从未视觉验证"
  - ProofShot（826 stars）专门解决"AI Agent 眼睛"问题，HN 161分，验证市场需求
  - 技术栈可行：image-rs + clap + dist，3-5MB 单二进制，性能绰绰有余
  - 产品形态确定：MCP Server（双模式 CLI）为最优形态，所有 AI Coding 工具通用
  - 策略调整：从"CLI 优先"转向"MCP 优先"
- 第4轮决策：放弃 MCP，改为 CLI-only + 登录制解锁远端 AI
  - 理由：Agent 用 Bash 调用 CLI 已是 Agent-native，MCP 增加维护负担但收益有限
  - 架构：L1 本地免费（inspect/overview/tile/viewport）+ L2 远端需 login（analyze/ocr/semantic-diff）
  - 商业化：天然免费/付费分层，跟 gh/vercel/stripe-cli 同模式
  - 技术栈精简：去掉 rmcp/tokio，二进制更小（3-5MB）

### 2026-06-01 — 项目初始化
- 产出：project.md + AGENTS.md + CLAUDE.md
- 技术决策：Rust + image-rs 0.25.x + clap 4.x + cargo-dist
- 架构：CLI-only 单二进制，L1 本地免费 + L2 远端 AI 需 login
- 共享决策：PD1-PD6（JSON-first、统一坐标、Agent-safe、稳定错误码、命名、二进制大小）
- Phase 3（fe-system）跳过：纯 CLI 工具无前端

### 2026-06-01 — Phase 1 需求定义
- 产出：docs/features/phase1-viewport-commands/PRD.md（6 个用户故事，18 个验收条件）
- 核心命令：inspect(P0) / overview(P0) / tile(P0) / viewport(P0, 3 种模式) / resize(P1) / rotate(P1)
- 统一坐标系：rect / percent / anchor + 坐标映射
- Agent-safe：不覆盖源文件、路径 sandbox、像素限制 100MP、tile 限制 64
- 范围排除：diff、concat/blur、login、远端 AI、MCP、递归探索

### 2026-06-01 — Phase 1 技术详设
- 产出：docs/features/phase1-viewport-commands/contract.md
- 7 个共享决策（FD1-FD7）：workspace 结构、坐标映射、输出格式、安全校验、tile 余数、格式推断、策略建议
- 完整数据模型：Point/Rect/Percent/Anchor/CommandResult<T>/CoordinateMapping + 6 个命令输出类型
- 10 个模块：types / guard / coord / inspect / overview / tile / viewport / resize / rotate / main
- 错误码枚举：9 个稳定错误码（FILE_NOT_FOUND → PIXEL_LIMIT_EXCEEDED）
- 跳过 API/DB/Frontend 领域（纯 CLI）

### 2026-06-01 — Phase 1 任务分解
- 产出：plan.md（9 个任务）+ testing/test-cases.md
- 关键路径：01(workspace) → 02(guard+coord) → 03(inspect) → 06(viewport) → 07(resize) → 09(CLI 集成)
- 并行机会：Task-03/04/05（P0 三命令）、Task-07/08（P1 两命令）
- 预估：~7.5h
- 检查点：CP-1(guard/coord) / CP-2(P0 四命令) / CP-3(CLI+二进制大小)

### 2026-06-01 — Phase 1 测试策略
- 产出：testing/contract.md（T1-T5）+ testing/test-cases.md（~60 用例）
- 测试类型：70% 单元 + 25% 集成 + 5% schema 快照
- 关键路径全覆盖：guard → coord → inspect → overview → tile → viewport
- Mock：无（纯本地库 + tempdir 隔离）
- CI：fmt + clippy + test + release 大小检查

### 2026-06-01 — Phase 1 构建完成
- 产出：9 个任务全部实现（Task-01 到 Task-09）
- 模块：types / guard / coord / inspect / overview / tile / viewport / resize / rotate + CLI main.rs
- 测试：61 passed（47 单元 + 14 集成）
- Clippy：0 warnings（修复 doc comment /// → //!，uninlined_format_args）
- Fmt：clean
- Release 二进制：5.1MB（≤8MB 约束）
- 命令可用：inspect / overview / tile / viewport(anchor|percent|rect) / resize / rotate
- JSON 输出验证：所有命令正确输出 CommandResult<T> 结构
- 坐标映射：viewport/overview/resize/rotate 均输出 coordinate_mapping

### 2026-06-01 — 端到端验证
- 测试集：7 张真实无版权图片（picsum.photos/Unsplash），存入 fixtures/e2e/
  - landscape_large.jpg (3200×2400) — 触发 needs_overview
  - portrait_tall.jpg (1200×3000) — 竖长图极端比例
  - panorama_wide.jpg (4000×1500) — 宽幅全景
  - urban_square.jpg (2000×2000) — 正方形，9 锚点全覆盖测试
  - screenshot_like.jpg (1920×1080) — 模拟 App 截图（header/sidebar 裁剪）
  - nature_small.jpg (400×300) — 小图（不触发 overview）
  - nature_small.png (400×300) — PNG 格式测试
- 40 个输出文件全部为有效图片
- 坐标验证：anchor 九宫格、percent 百分比、rect 像素三种模式计算正确
- tile 余数策略验证：3200/3=1066+1066+1068、4000/3=1333+1333+1334
- resize bug 修复：forced resize 用 resize_exact 替代 thumbnail（保持比例问题）
- 错误场景全通过：path_escape / output_same_as_input / invalid_coordinates / invalid_parameters

### 2026-06-01 — Skills + License + README + Plugin
- 产出：skills/ 目录（1 个 Claude Code skill + 1 个 Cursor rule + 1 个 Codex AGENTS.md）
- Skill：vistools（统一 skill，涵盖 inspect/overview/tile/viewport/resize/rotate 全部工作流）
- License：LICENSE-MIT + LICENSE-APACHE（双许可）
- README.md：完整文档（命令参考、JSON 示例、workflow、安装说明）
- Plugin：`.claude-plugin/plugin.json` + `hooks.json`，支持 `/plugin install` 一键安装
- Setup hook：`bin/install.sh` 自动编译 CLI 并安装到 PATH
- 覆盖平台：Claude Code（plugin 一键安装）、Cursor（.mdc rule）、Codex（AGENTS.md）

### 2026-06-01 — 全局重命名 image-viewport → vistools + 简化安装
- 二进制名：`image-viewport` → `vistools`
- Crate 名：`image-viewport-core` → `vistools-core`，`image-viewport` → `vistools`
- Skill 目录：`skills/claude-code/image-viewport/` → `skills/claude-code/vistools/`
- Cursor rule：`image-viewport.mdc` → `vistools.mdc`
- Plugin name：`vistools`，skill 调用：`/vistools screenshot.png`
- 简化安装：删除 `bin/install.sh` + `hooks.json`，改为 `cargo install --path crates/cli`（安装到 `~/.cargo/bin/`）
- 更新范围：Cargo.toml、main.rs、integration_test.rs、plugin.json、SKILL.md、README.md、AGENTS.md、project.md、contract.md
- 61 tests 全部通过，clippy 0 warnings

### 2026-06-01 — 拆分为两个仓库 + CI 同步 + 内置 binary
- 主仓库 `vistools`：Rust 源码 + skills + CI workflow（source of truth）
- 分发仓库 `vistools-skills`：skill 文件 + 预编译 binary（用户安装）
- `bin/vistools`：平台检测 launcher（自动选 macos-arm64 / macos-x64 / linux-x64）
- CI workflow：push 到 main → 编译 3 平台 binary → 同步到 vistools-skills
- plugin.json 声明 `"bin": "./bin/"`，安装后 `vistools` 自动加到 PATH
- 用户只需 `/plugin install https://github.com/ZeroZ-lab/vistools-skills`，无需手动安装 CLI

### 2026-06-01 — 拆分为两个仓库
- 主仓库 `vistools`：只保留 Rust 源码（crates/）、测试图片（fixtures/）、设计文档（docs/）
- 新仓库 `vistools-skills`：AI agent skills（Claude Code plugin + Cursor rule + Codex instructions）
- 理由：plugin install 时用户只需几 KB 的 skill 文件，不应下载整个 Rust 项目（含 fixtures ~3MB）
- 从 vistools 移除：`skills/`、`.claude-plugin/`
- README 更新：Skills 章节指向 vistools-skills 仓库

### 2026-06-01 — Code Review 修复（5 Critical + 7 Warning + 5 Nit）
- C3: fixture 路径改用 `std::env::var("CARGO_MANIFEST_DIR")`（运行时），集中到 `test_support.rs`，任何人 fork 均可运行
- C1/C2: 消除 5 处生产 `unwrap()`（overview/tile/viewport/resize/rotate 的 `fs::metadata`），改为 `match → CommandResult::err`
- C5: CLI 退出码从字符串匹配 JSON 改为 `(json, ok)` 元组模式
- C4: `DEFAULT_JPEG_QUALITY=95` 从死代码变为实际使用（`JpegEncoder::new_with_quality`）
- W1: 新建 `util.rs` 共享 `save_image`，所有命令统一 JPEG 质量
- W3: 修复 `cargo fmt` 违规
- W4: viewport anchor 模式超源图时添加 warning
- W5: tile 拒绝 `cols > src_w` / `rows > src_h`（防 0 宽度 tile）
- W7: overview 拒绝 `max_width=0`
- N4: AGENTS.md `vistolls/` → `vistools/` 拼写修复
- 新增 3 个测试：`overview_rejects_zero_max_width` / `tile_rejects_cols_exceeding_width` / `viewport_warns_when_larger_than_source`
- 测试结果：64 tests passed（50 单元 + 14 集成），clippy 0 warnings，fmt clean
- Release 二进制：5.2MB（≤8MB 约束）

### 2026-06-01 — Phase 1 命令边界收敛
- 产品定位：从通用图片处理收束为 LLM 视觉仪器层（视野导航 + 坐标映射）
- 公开命令：保留 inspect / overview / tile / viewport
- 移除命令：resize / rotate（通用像素处理能力，不进入当前公开命令面）
- overview：`--max-width` 改为 `--max-side`，按长边缩放，适配竖长截图/设计稿
- 文档同步：project.md、PRD、contract、plan、testing、README、Skill、AGENTS
- 下一步升级：zoom（显微镜）/ sample（取色器）/ grid（坐标纸）/ diff（变化感知）

### 2026-06-01 — Roadmap 文档
- 产出：docs/roadmap.md
- 定位：LLM 视觉仪器层，而不是通用图片处理库
- 阶段：v0.2 稳定 4 个视野核心命令；v0.3 做 sample / zoom / grid；v0.4 做 diff / lens / measure；v0.5 做 Agent 工作流封装
- 推荐顺序：percent 越界校验与 schema 快照 → sample → zoom → grid → diff → lens / measure
- 暂缓：screenshot、OCR、MCP server、远端 AI 分析

### 2026-06-01 — v0.2 核心稳定补丁
- inspect：新增 `recommended_next`、`reason`、`suggested_max_side`，让 Agent 直接读取下一步建议
- viewport percent：严格拒绝 NaN、超出 0..1 的参数，以及 `x + w` / `y + h` 越界
- 错误码：参数范围错误返回 `INVALID_PARAMETERS`，区域越界返回 `INVALID_COORDINATES`
- 测试：新增 inspect 大图推荐、percent 超范围、percent 区域溢出、NaN 场景
- Roadmap：v0.2 剩余项收敛为 schema 快照

### 2026-06-01 — Schema 快照测试
- 新增：crates/cli/tests/schema_snapshot_test.rs
- 覆盖：inspect 成功、错误信封、overview、tile、viewport 的 CLI JSON 输出形状
- 方法：把动态值归一化为类型形状，比较内联 JSON 快照
- 目的：锁住 Agent 解析契约，后续新增 sample/zoom/grid 时避免破坏现有 schema

### 2026-06-01 — sample 取色器
- 新增：`vistools sample`，第一批视觉仪器中的取色器
- 模式：点取色 `--x/--y`；区域取色 `--rect x,y,width,height`
- 输出：RGBA/RGB/HEX/alpha、区域平均色、alpha min/max/average/transparent_ratio、pixel_count
- 错误边界：缺失/冲突模式和 malformed rect → `INVALID_PARAMETERS`；越界 → `INVALID_COORDINATES`；零尺寸 rect → `INVALID_DIMENSIONS`
- 测试：新增 sample core 单测、CLI 集成测试和 point/rect schema 快照

### 2026-06-01 — v0.2.0 发布
- 版本：0.1.2 → 0.2.0（breaking：移除 resize/rotate 命令）
- 自 v0.1.2 以来累计变更：
  - 命令面收敛：保留 inspect / overview / tile / viewport，移除 resize / rotate
  - v0.2 核心稳定：inspect recommended_next + percent 越界校验 + schema 快照
  - v0.3 sample：点/区域取色器 + alpha 统计
  - 文档：roadmap.md（LLM 视觉仪器层路线图）
- 测试：77 passed（52 单元 + 18 集成 + 7 schema 快照），clippy 0 warnings

### 2026-06-02 — v1 原子命令面收敛
- 产出：docs/features/v1-agent-command-surface/contract.md + changelog.md
- 结论：`vistools` v1 收敛为三层命令面：视野层（inspect/overview/tile/viewport）、测量层（sample/diff/measure）、断言层（assert-color/assert-diff/assert-region）
- 明确排除：行业业务命令、通用像素处理、OCR/检测/分割、内建 orchestrator、自然语言报告
- 产品定位进一步收紧：不是图像处理库，而是 Agent 的视觉测量命令面 + 坐标化证据层

### 2026-06-02 — 摄影计量能力探索
- 产出：docs/features/photography-metering/idea-brief.md（4 个方向，1 轮收敛，推荐：影调计量为核心）
- 核心结论：在现有 photo.rs 6 个命令基础上，扩展摄影语言级计量能力
- P0 MVP：histogram 增强（+RGB 通道）+ zone-map（Zone System 分区）+ exposure（EV 估算 + 测光模式）
- P1 扩展：focus-map（焦点地图）+ white-balance（色温估算）
- P2 进阶：gamut（色域检查）+ noise（噪声估算）
- 目标场景：Agent 帮摄影师批量质检照片

### 2026-06-02 — 摄影计量初始化
- 产出：project.md 更新（PD7-PD9）+ AGENTS.md 同步 v0.2.3 结构
- PD7：摄影计量算法——纯像素数学，不加新依赖，手写 sRGB→Lab 变换
- PD8：模块归属——全部在 photo.rs 内扩展，不加新模块
- PD9：histogram 向后兼容——--rgb flag 增量输出
- AGENTS.md 同步：反映 v0.2.3 重构后的实际目录结构（core 9 模块 + CLI commands/ 分层）
- Phase 3 跳过：纯 CLI 无前端

### 2026-06-02 — 摄影计量需求定义
- 产出：docs/features/photography-metering/PRD.md（3 个用户故事，19 个验收条件）
- US-01: histogram --rgb（AC-01-1~4：RGB 三通道直方图 + 向后兼容 + rect + clipping）
- US-02: zone-map（AC-02-1~5：11 区分布 + 纯黑/纯白边界 + rect + 代表区域映射）
- US-03: exposure（AC-03-1~10：4 种测光模式 + assessment 三档 + 参数校验 + rect + 边界值）
- 范围排除：gamma 反转 Zone / RGB Zone / EV 绝对值 / 自动建议 / P1+ 命令

### 2026-06-02 — 摄影计量详设
- 产出：docs/features/photography-metering/contract.md（FD1-FD6）
- FD1: histogram --rgb 增量输出（Option<RgbHistogram>，不传时无 rgb 字段）
- FD2: Zone System 线性 11 区（luma * 11 / 256，Zone V = 116-139 ≈ 118 中灰）
- FD3: EV = log2(weighted_mean_luma / 118.0)
- FD4: 4 种测光模式（evaluative / spot / center-weighted Gaussian / highlight-weighted top 10%）
- FD5: assessment 三档（under < -0.5 / correct -0.5..0.5 / over > 0.5）
- FD6: CLI 参数（histogram 新增 HistogramArgs + --rgb bool；zone-map 复用 RegionArgs；exposure 新增 ExposureArgs + --mode/--x/--y）
- 新增类型：RgbHistogram / ChannelHistogram / ZoneMapOutput / ZoneInfo / ExposureOutput
- 修改类型：HistogramMetrics 新增 rgb: Option<RgbHistogram>
- Phase 1-3 跳过：纯 CLI，无前端/后端/数据库

### 2026-06-02 — 摄影计量任务分解
- 产出：docs/features/photography-metering/plan.md（4 个任务）+ testing/test-cases.md（23 个用例骨架）
- Task-01: histogram --rgb 增强（修改现有命令，风险最高先验证）
- Task-02: zone-map 新命令（新命令，与 Task-03 并行）
- Task-03: exposure 新命令（4 种测光模式，与 Task-02 并行）
- Task-04: schema snapshot + 集成验证（全量回归）
- 关键路径：Task-01 → Task-02/03 → Task-04
- 预估：~4h

### 2026-06-02 — 摄影计量测试策略
- 产出：docs/features/photography-metering/testing/contract.md（T1-T5）
- T1: 单元 65% + 集成 20% + Schema 快照 15%
- T2: 关键路径全覆盖（zone_index / weighted_luma / EV / 4 种测光模式）
- T3: tempfile 临时图（纯色/渐变/精确亮度），无共享 fixture
- T4: 无 Mock（纯本地库，全部真实调用）
- T5: cargo test + clippy + fmt，失败阻断合并

### 2026-06-02 — 摄影计量 P0 构建完成
- 产出：histogram --rgb / zone-map / exposure 三个命令实现 + 35 个新测试
- histogram --rgb：R/G/B 三通道直方图 + 分位数 + 通道级 clipping（AC-01-1~4）
- zone-map：Zone System 11 区分布 + 代表区域坐标（AC-02-1~5）
- exposure：EV 估算 + 4 种测光模式 + assessment 三档（AC-03-1~10）
- 单元测试：19 个（photo.rs）
- 集成测试：12 个（CLI 参数解析 + JSON 输出 + 错误处理）
- Schema snapshot：4 个（histogram --rgb / zone-map / exposure / exposure spot）
- 向后兼容：histogram 不带 --rgb 时输出不变（无 rgb 字段）
- 验证：117 tests passed（82 core + 35 integration），clippy 0 warnings，fmt clean

### 2026-06-02 — 摄影计量验收阶段定义
- 产出：docs/features/photography-metering/acceptance.md
- 决策：P0 build 后先做真实照片验收，不直接进入 P1（focus-map / white-balance）
- 验收目标：
  - 验证 `histogram --rgb` 是否比亮度直方图提供增量判断信息
  - 验证 `zone-map` 在真实照片上的分区语义是否稳定
  - 验证 `exposure.assessment` 与人工曝光标签一致率是否达到 80%
- 输出：样本要求、执行命令、人工记录模板、通过/不通过后的决策出口

### 2026-06-02 — 摄影计量 P1 focus-map 实现
- 触发：用户明确要求直接开始写代码，跳过“先完成真实照片验收再进入 P1”的建议顺序
- 新增命令：`vistools focus-map <INPUT> --rows <N> --cols <M>`
- 输出：NxM 网格锐度矩阵、`best_cell`、`focus_point`
- 实现方式：在 `photo.rs` 内复用现有 sharpness 核，按 tile 式 remainder 策略切分 cell
- CLI：新增 `FocusMapArgs` 与 `focus-map` 子命令注册
- 测试：新增 core 单测、CLI 集成测试、schema snapshot
- 验证：141 tests passed（86 core + 38 integration + 17 schema），clippy 0 warnings，fmt clean

### 2026-06-02 — v0.2.3 架构扩展性重构
- 版本：0.2.2 → 0.2.3
- core 拆分：`types` 历史聚合层收敛为 `constants` / `error` / `geom` / `protocol` / `source` / `region`
- 命令实现切到统一基础设施：`overview` / `tile` / `viewport` / `sample` / `photo` 复用统一 source 加载与 region 校验
- CLI 拆分：`main.rs` 仅保留注册和 dispatch；新增 `commands/` 与 `parse.rs`
- 协议收敛：
  - `CoordinateMapping` 从 `crop_origin_in_source + scale_factor` 改为 `source_origin + scale_x + scale_y`
  - `ViewportOutput.crop` 从动态 `mode + params` 改为强类型 `CropSpec`
- 正确性修复：anchor 模式超源图请求时，输出 `crop.region`、实际结果尺寸和 mapping 保持一致
- 验证：104 tests passed（67 core 单测 + 25 CLI 集成 + 12 schema snapshot），fmt clean，clippy 0 warnings
