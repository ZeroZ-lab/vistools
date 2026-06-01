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
