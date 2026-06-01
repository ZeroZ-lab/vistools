### v1.0 — 2026-06-01 — 需求定义

- **触发**：用户要求 define Phase 1 功能
- **产出**：PRD.md（6 个用户故事，18 个验收条件）
- **范围排除**：diff/compare、concat/blur、login/远端 AI、MCP、递归视野探索
- **优先级排序**：P0 = inspect/overview/tile/viewport，P1 = resize/rotate

### v1.0 — 2026-06-01 — 技术详设

- **触发**：用户要求 detail Phase 1
- **产出**：contract.md（7 个 FD 决策 + 完整数据模型 + 模块索引）
- **Workspace**：image-viewport-core (lib) + image-viewport (bin)
- **核心模块**：types / guard / coord / inspect / overview / tile / viewport / resize / rotate
- **跳过领域**：API / DB / Frontend（纯 CLI 无需）

### v1.0 — 2026-06-01 — 任务分解

- **触发**：用户要求 plan Phase 1
- **产出**：plan.md（9 个任务）+ testing/test-cases.md（P0 用例骨架）
- **执行顺序**：01(workspace) → 02(guard+coord) → 03+04+05 并行(inspect/overview/tile) → 06(viewport) → 07+08 并行(resize/rotate) → 09(CLI 集成)
- **关键路径**：01 → 02 → 03 → 06 → 07 → 09
- **预估总时间**：~7.5h
- **检查点**：3 个（guard/coord 验证、P0 四命令可用、CLI 集成 + 二进制大小）

### v1.0 — 2026-06-01 — 测试策略

- **触发**：用户要求 test Phase 1
- **产出**：testing/contract.md（T1-T5）+ testing/test-cases.md（完整用例）
- **T1 测试类型**：70% 单元 + 25% 集成 + 5% schema 快照
- **T2 覆盖**：P0 全覆盖 + P1 正常+错误 + CLI 冒烟
- **T3 数据**：小 fixture 提交 + 大 fixture CI 生成
- **T4 Mock**：无 Mock（纯本地库）+ tempdir 隔离
- **T5 CI**：fmt + clippy + test + 二进制大小检查
- **用例数**：~60 个测试用例（正常+边界+错误全覆盖）

### v1.1 — 2026-06-01 — 命令边界收敛

- **触发**：用户确认“先精简，再升级”，产品定位收束为 LLM 视觉仪器层
- **产出**：代码与文档同步为 4 个公开命令：inspect / overview / tile / viewport
- **移除**：公开 resize / rotate；理由是二者属于通用像素处理库能力，不属于视野导航核心
- **契约变更**：overview 参数从 `--max-width` 改为 `--max-side`，按源图长边生成模型可读总览
- **后续升级方向**：zoom / sample / grid / diff / lens / measure

### v1.2 — 2026-06-01 — Roadmap 文档

- **触发**：用户要求“下一步先写我们的 roadmap 到文档”
- **产出**：docs/roadmap.md（v0.2-v0.5 阶段路线 + 推荐实现顺序 + 暂缓项）
- **决策**：下一步先稳核心，再按 sample → zoom → grid → diff → lens/measure 升级
- **边界**：继续不做通用 resize/rotate/convert、修图滤镜、OCR、MCP server、远端 AI

### v1.3 — 2026-06-01 — v0.2 核心稳定补丁

- **触发**：用户确认继续开发 v0.2 core stabilization
- **产出**：inspect 增加 `recommended_next` / `reason` / `suggested_max_side`
- **修复**：viewport percent 严格校验 0..1、区域溢出和 NaN，避免隐式 clamp
- **测试**：新增大图推荐、percent 超范围、percent 区域越界、NaN 用例

### v1.4 — 2026-06-01 — Schema 快照测试

- **触发**：用户确认继续按 roadmap 执行
- **产出**：crates/cli/tests/schema_snapshot_test.rs
- **覆盖**：inspect 成功、错误信封、overview、tile、viewport 的 CLI JSON 输出形状
- **决策**：不引入 snapshot 依赖，使用内联结构快照，避免动态路径/耗时造成噪声

### v1.5 — 2026-06-01 — sample 取色器

- **触发**：用户要求实现 `vistools sample` 作为第一批视觉仪器
- **产出**：`core::sample` + CLI `sample` 命令 + point/rect 输出类型
- **能力**：点取色返回 RGBA/RGB/HEX/alpha；区域取色返回平均色、alpha 统计和 pixel_count
- **错误码**：非法模式/rect 语法为 `INVALID_PARAMETERS`，越界为 `INVALID_COORDINATES`，零尺寸 rect 为 `INVALID_DIMENSIONS`
- **测试**：新增 core 单测、CLI 集成测试、sample point/rect schema 快照
