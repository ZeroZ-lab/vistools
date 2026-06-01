# project.md — vistools

> 项目级技术决策 + 共享约束。Feature 级文档引用本文件，不重复。

## 版本信息

| 项目 | 值 |
|------|-----|
| 版本 | v1.0 |
| 日期 | 2026-06-01 |
| 团队 | Zheng Jianqiao |

---

## 业务目标

**问题**：AI Agent（Claude Code / Cursor / Codex）面对大图（截图、设计稿、长网页）时，缺少一个可编程的视觉视野控制层。大图直接发给视觉模型导致：成本高（tokens = w×h/750）、细节丢失（>1568px 自动缩放）、注意力被稀释（边角细节忽略）。Claude Code 80+ GitHub issues 验证此痛点。

**目标用户**：

| 角色 | 描述 | 核心诉求 |
|------|------|---------|
| AI Coding 用户 | Claude Code / Cursor / Codex 用户 | Agent 改完前端代码后能视觉验证 UI |
| 前端开发团队 | 需要视觉回归 + Agent 修复闭环 | 不只写代码，还能检查结果 |
| 设计系统团队 | 关心设计还原和 UI 一致性 | 从设计稿截图分区域理解细节 |
| Agent 开发者 | Computer-use / Browser Agent 开发者 | 截图预处理、隐私清洗、坐标映射 |

**MVP 范围（Phase 1）**：
- 必须做：inspect / overview / tile / viewport / resize / rotate，JSON 输出 + 坐标映射
- 不做：远端 AI 功能（Phase 3）、登录系统（Phase 2）、diff/compare（Phase 2）、MCP server、IDE 插件、OCR/版面分析

**成功指标**：

| 指标 | 目标值 | 评估时间 |
|------|--------|---------|
| H1: Agent 主动调用率 | ≥50% 前端修改任务 | 2 周 |
| H2: 局部裁剪识别提升 | ≥1.3x vs 整图 | 1 周 |
| H3: 付费意愿 | ≥30% 活跃用户 | 4 周 |
| 早期用户反馈 | ≥3 人说"有用" | 6 周 |

---

## 架构模式

**选择**：CLI-only 单二进制，L1 本地免费 + L2 远端 AI 需 login
**理由**：Agent 天然会用 Bash 跑命令（Claude Code / Cursor / Codex 都支持），JSON 输出的 CLI 本身就是 Agent-native。不需要 MCP 中间层。与 `gh` / `vercel` / `stripe-cli` 同模式：免费本地 + 登录制解锁远端。
**拒绝**：MCP Server（增加维护负担，CLI JSON 已是 Agent-native）、Web App（脱离 Agent 上下文）、IDE 插件（分发摩擦大）

---

## 技术选型

| 层 | 选择 | 版本 | 理由 | 被拒方案 |
|----|------|------|------|---------|
| 语言 | Rust | 1.94+ | 性能、单二进制分发、与 imgctl 差异化 | — |
| 图片处理 | image-rs (`image` crate) | 0.25.x | 纯 Rust、PNG/JPEG/WebP/TIFF/AVIF、resize/crop/blur 全支持 | libvips（无 Rust 绑定）、turbojpeg（C 依赖） |
| CLI 框架 | clap | 4.x | Rust 标准 CLI 库 | — |
| EXIF 解析 | kamadak-exif | — | EXIF 字段提取 | — |
| HTTP 客户端 | reqwest | — | Phase 3 调用远端 AI API | — |
| 认证 | OAuth2 + 本地 token 存储 | — | Phase 2 login 流程 | — |
| 分发 | cargo-dist (`dist`) | — | 多平台 CI + GitHub Releases + Homebrew | — |
| 测试 | Rust 内置 + assert_cmd | — | CLI E2E 测试 | — |

---

## 共享决策

| # | 决策 | 选择 | 详情 |
|---|------|------|------|
| PD1 | 输出格式 | JSON-first | 所有命令默认 `--json` 输出结构化 JSON，可选 `--quiet` 只输出错误码 |
| PD2 | 坐标系统 | 统一坐标系 | 原点左上角，x→右，y→下。rect = (x, y, w, h)，percent = 0.0-1.0，anchor = 九宫格语义。所有操作输出包含坐标映射 |
| PD3 | 文件安全 | Agent-safe | 不覆盖源文件、输出到指定路径、限制最大 tile 数（64）、限制最大像素数（100MP）、错误也返回 JSON |
| PD4 | 错误处理 | 稳定错误码 | 每种错误有 `error.code`（如 `UNSUPPORTED_FORMAT`），Agent 可 pattern-match |
| PD5 | 命名规范 | 二进制名 `vistools` | 子命令：inspect / overview / tile / viewport / resize / rotate / concat / blur / diff / analyze / ocr / semantic-diff / login / whoami / logout |
| PD6 | 二进制大小 | ≤8MB（Phase 1） | 纯 Rust 无 C 依赖，release LTO + strip。对比 ImageMagick 30-50MB |

---

## 共享约束

### 安全

- 不覆盖源文件（所有输出必须指定路径）
- 路径 sandbox：拒绝 `..` 和绝对路径逃逸
- 限制最大 tile 数（64）和最大像素数（100MP）
- Phase 3+：API token 存储在 `~/.vistools/config.json`，权限 600

### 性能

- inspect（读元数据）：<1ms
- viewport/crop：<5ms
- overview/resize：<200ms
- tile（单个 tile）：<5ms

### 兼容性

- JSON 输出 schema 向后兼容：新增字段可选，不删已有字段
- CLI 参数不删不改（只加新参数）
- 子命令名稳定，不重命名

### 工程约束

**模块边界**：
- Workspace: `vistools-core`（library）+ `vistools`（CLI binary）
- core 导出公共 API，CLI binary 只做参数解析 + 调用 core
- 未来远端 AI 功能为可选 feature flag（`remote`）

**类型不变量**：
- 坐标用 newtype（`Point`、`Rect`、`Percent`、`Anchor`），不用裸 tuple
- 图片尺寸用 `Size { width: u32, height: u32 }`
- 所有命令输出统一 `CommandResult<T>` 泛型

**测试策略**：
- 单元测试：core 库每个命令（crop/tile/resize/viewport 坐标计算）
- 集成测试：assert_cmd E2E 测试 CLI 参数解析 + JSON 输出 schema
- 测试图片：小尺寸 fixture（64x64、256x256、1000x1000）
- 目标：≥80% 行覆盖率

**lint / format / CI**：
- rustfmt + clippy（deny warnings）
- cargo-dist 自动化 GitHub Releases（macOS ARM/Intel、Linux x64/ARM64、Windows x64）
- CI: test + clippy + fmt check

---

## Feature 索引

> 由 detail 阶段自动维护，不手动预填。每次 detail 完成后同步更新。

| Feature | 目录 | 状态 | 说明 |
|---------|------|------|------|
| Phase 1: 视野控制命令集 | docs/features/phase1-viewport-commands/ | ③详设完成 | 6 命令 + FD1-FD7 + 完整数据模型 |