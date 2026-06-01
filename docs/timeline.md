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
- 主仓库 `vistolls`：Rust 源码 + skills + CI workflow（source of truth）
- 分发仓库 `vistools-skills`：skill 文件 + 预编译 binary（用户安装）
- `bin/vistools`：平台检测 launcher（自动选 macos-arm64 / macos-x64 / linux-x64）
- CI workflow：push 到 main → 编译 3 平台 binary → 同步到 vistools-skills
- plugin.json 声明 `"bin": "./bin/"`，安装后 `vistools` 自动加到 PATH
- 用户只需 `/plugin install https://github.com/zhengjianqiao/vistools-skills`，无需手动安装 CLI

### 2026-06-01 — 拆分为两个仓库
- 主仓库 `vistolls`：只保留 Rust 源码（crates/）、测试图片（fixtures/）、设计文档（docs/）
- 新仓库 `vistolls-skills`：AI agent skills（Claude Code plugin + Cursor rule + Codex instructions）
- 理由：plugin install 时用户只需几 KB 的 skill 文件，不应下载整个 Rust 项目（含 fixtures ~3MB）
- 从 vistolls 移除：`skills/`、`.claude-plugin/`
- README 更新：Skills 章节指向 vistolls-skills 仓库
