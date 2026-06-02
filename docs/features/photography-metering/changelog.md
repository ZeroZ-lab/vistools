# Changelog — 摄影计量

### v0.3.0 — 2026-06-02 — 探索 + 初始化 + 需求定义 + 详设

- **触发**：用户要求增加摄影行业计量能力
- **探索产出**：idea-brief.md（4 个方向，1 轮收敛，推荐：影调计量核心）
- **初始化产出**：project.md 更新 PD7-PD9 + AGENTS.md 同步 v0.2.3 结构
- **需求产出**：PRD.md（3 个用户故事，19 个验收条件）
  - US-01: histogram --rgb（4 个 AC）
  - US-02: zone-map（5 个 AC）
  - US-03: exposure（10 个 AC）
- **详设产出**：contract.md（FD1-FD6 + 共享数据模型 + 模块索引 + 代码映射）
  - FD1: histogram --rgb 增量输出策略
  - FD2: Zone System 线性 11 区映射（luma * 11 / 256）
  - FD3: EV = log2(luma / 118)
  - FD4: 4 种测光模式加权
  - FD5: assessment ±0.5 EV 边界
  - FD6: CLI 参数模式（复用 RegionArgs + 新增 ExposureArgs）
- **范围排除**：gamma 反转 Zone / RGB Zone / EV 绝对值 / 自动建议 / P1+ 命令 / 批量 / 报告
- **决策**：PD7（纯像素数学）PD8（photo.rs 内扩展）PD9（histogram 向后兼容）

### v0.3.0 — 2026-06-02 — 任务分解

- **触发**：用户确认进入 plan 阶段
- **产出**：plan.md（4 个任务）+ testing/test-cases.md（23 个用例骨架）
- **任务**：
  - Task-01: histogram --rgb 增强（修改现有命令，向后兼容风险最高，优先验证）
  - Task-02: zone-map 新命令（与 Task-03 可并行）
  - Task-03: exposure 新命令（与 Task-02 可并行）
  - Task-04: schema snapshot + 集成验证
- **关键路径**：Task-01 → Task-02/03 → Task-04
- **检查点**：CP-1（histogram 向后兼容验证）+ CP-2（全量测试通过）

### v0.3.0 — 2026-06-02 — 测试策略

- **触发**：用户确认进入 test 阶段
- **产出**：testing/contract.md（T1-T5）
- **T1 类型**：单元 65% + 集成 20% + Schema 快照 15%
- **T2 覆盖**：关键路径全覆盖（zone_index / weighted_luma / EV 计算），≥80% 行覆盖率
- **T3 数据**：tempfile 临时图（纯色/渐变/精确亮度），无共享 fixture
- **T4 Mock**：无 Mock（纯本地库，全部真实调用）
- **T5 CI**：test + clippy + fmt，失败阻断合并

### v0.3.0 — 2026-06-02 — P0 构建完成

- **触发**：用户确认进入代码生成阶段
- **产出**：3 个命令实现 + 35 个新测试 + 全部 PRD AC 验证
- **histogram --rgb**（AC-01-1~4）：
  - R/G/B 三通道 256 bins + mean/p05/p50/p95
  - 通道级 clipping（≤5 和 ≥250 像素计数）
  - 向后兼容：不传 --rgb 时无 rgb 字段
- **zone-map**（AC-02-1~5）：
  - Zone System 线性 11 区（Zone 0-X）
  - 每区 pixel_count/ratio/representative_rect
  - 罗马数字标签（0/I/II/.../X）
- **exposure**（AC-03-1~10）：
  - 4 种测光模式：evaluative / spot / center-weighted / highlight-weighted
  - EV 计算：log2(weighted_luma / 118)
  - Assessment 三档：under (< -0.5) / correct (±0.5) / over (> 0.5)
  - Spot 模式验证 --x/--y 必须提供
- **测试**：
  - 单元测试 19 个（photo.rs）
  - 集成测试 12 个（CLI 参数 + JSON 输出 + 错误处理）
  - Schema snapshot 4 个（锁定 JSON 形状）
- **验证**：117 tests passed / clippy 0 warnings / fmt clean

### v0.3.0 — 2026-06-02 — 验收阶段定义

- **触发**：用户确认进入下一个合理阶段
- **产出**：acceptance.md（真实照片验收目标 + 样本要求 + 通过标准 + 决策出口）
- **结论**：P0 build 后先做真实图片验收，不直接进入 P1 命令扩展
- **通过标准**：
  - `exposure.assessment` 与人工标签一致率 ≥ 80%
  - `histogram --rgb` 在至少 30% 样本中提供增量判断信息
  - `zone-map` 无系统性违背直觉的主分区结果
- **下一步**：
  - 验收通过 → 进入 `focus-map / white-balance` define
  - 验收不通过 → 先修 P0，不扩命令面

### v0.3.0 — 2026-06-02 — 验收执行模板

- **触发**：用户要求继续推进验收阶段
- **产出**：
  - `acceptance-commands.md`（前置验证 + fixture 冒烟 + 局部检查 + 真实照片命令模板）
  - `acceptance-log-template.md`（汇总表 + 单张照片记录模板 + 统计规则）
- **目的**：把验收从“描述性流程”推进到“可直接执行和记账”的状态

### v0.3.0 — 2026-06-02 — Fixture 冒烟验收

- **触发**：用户要求继续执行验收
- **执行**：
  - 跑通 `cargo test`
  - 跑通 `cargo clippy -- -D warnings`
  - 跑通 `cargo fmt -- --check`
  - 对 `landscape_large` / `portrait_tall` / `screenshot_like` / `urban_square` 执行 histogram / zone-map / exposure 抽样验收
- **产出**：`acceptance-fixture-log.md`
- **发现**：
  - 文档中的 `--json` 示例错误；摄影计量命令默认输出 JSON，不接受 `--json`
  - `highlight-weighted` 与 `evaluative` 在高对比样本上产生可解释分歧，说明模式差异有效
  - histogram 不带 `--rgb` 时仍无 `rgb` 字段，向后兼容成立

### v0.3.1 — 2026-06-02 — P1 focus-map

- **触发**：用户明确要求直接开始实现代码
- **备注**：这一步跳过了“先完成真实照片验收再进入 P1”的推荐顺序，属于用户驱动的范围前移
- **产出**：
  - `focus-map` 新命令
  - `FocusMapOutput` / `FocusCell` 协议类型
  - CLI `FocusMapArgs` + 子命令注册
- **能力**：
  - 输入：`<INPUT> --rows N --cols M [--rect x,y,width,height]`
  - 输出：每个 cell 的 `region + sharpness`、全图最清晰 `best_cell`、可继续深挖的 `focus_point`
  - 网格策略：复用 tile remainder 规则，最后一行/列吸收余数像素
- **验证**：
  - core 单测新增 4 个
  - CLI 集成测试新增 2 个
  - schema snapshot 新增 1 个
  - 全量验证：141 tests passed / clippy 0 warnings / fmt clean
