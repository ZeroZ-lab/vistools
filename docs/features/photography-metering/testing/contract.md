# Test Strategy — 摄影计量 P0

> 测试策略：纯像素数学 CLI，无外部依赖，无网络，无数据库。

## 版本信息

| 项目 | 值 |
|------|-----|
| 日期 | 2026-06-02 |
| 来源 | contract.md（FD1-FD6）+ plan.md（Task-01~04）+ test-cases.md |

---

## T1: 测试类型

> 引用 project.md 测试策略：单元测试 + 集成测试 + schema 快照。

| 类型 | 占比 | 目标 | 选择理由 |
|------|------|------|---------|
| 单元测试（core） | 65% | photo.rs 每个算法函数 | 纯像素数学，算法正确性是核心风险；执行快（<1ms/test） |
| 集成测试（CLI） | 20% | 每个命令的参数解析 + JSON 输出 + exit code | 验证 CLI 层参数传递和错误处理 |
| Schema 快照 | 15% | 每个命令的 JSON 输出形状 | 锁定 Agent 解析契约，后续变更不破坏兼容性 |

**被拒**：
- E2E 真实图片批量测试：P0 阶段不需要，留给验收阶段手动验证
- Property-based testing（proptest）：算法确定性高，基于例子的测试足够

---

## T2: 覆盖策略

> 引用 project.md：目标 ≥80% 行覆盖率。

**覆盖优先级**：关键路径全覆盖 > 边界值覆盖 > 错误路径覆盖

| 模块 | 可自动化 | 测试类型 | 覆盖程度 |
|------|---------|---------|---------|
| zone_index 映射 | ✅ | 单元 | 全覆盖（0/128/255 边界 + 中间值） |
| weighted_luma × 4 模式 | ✅ | 单元 | 全覆盖（每种模式至少 1 个正向 + 1 个边界） |
| EV 计算 + assessment | ✅ | 单元 | 全覆盖（under/correct/over 边界值） |
| execute_histogram --rgb | ✅ | 单元 | 全覆盖（rgb=true/false + rect + clipping） |
| execute_zone_map | ✅ | 单元 | 全覆盖（纯黑/纯白/渐变/rect） |
| execute_exposure | ✅ | 单元 | 全覆盖（4 种模式 + 错误参数） |
| CLI histogram --rgb | ✅ | 集成 | 正向（有 --rgb）+ 向后兼容（无 --rgb） |
| CLI zone-map | ✅ | 集成 | 正向 + rect |
| CLI exposure | ✅ | 集成 | 正向 + 错误参数（缺 --x/--y + 无效 mode） |
| Schema JSON 形状 | ✅ | 快照 | 每个命令 1 个 snapshot |

**被拒**：
- 100% 行覆盖率追求：zone_index 这种一行函数不值得为覆盖率单独写测试
- 性能基准测试：P0 阶段算法简单（<200ms），不需要 benchmark

---

## T3: 测试数据

| 数据类型 | 用途 | 构造方式 |
|---------|------|---------|
| 纯色图（64×64） | clipping / zone-map 边界 | ImageBuffer::from_pixel |
| 渐变图（256×256） | histogram / zone-map 正向 | ImageBuffer::from_fn，luma = x |
| 精确亮度图 | exposure EV 边界验证 | ImageBuffer::from_pixel（固定 luma 值：30/118/220） |
| 红/绿/蓝通道图 | RGB histogram 通道验证 | ImageBuffer::from_fn，单通道高其他低 |
| fixtures/e2e/ 真实图 | schema snapshot + 集成测试 | 已有 fixture |

**隔离原则**：
- 每个测试用 tempfile 创建临时 PNG，测试结束自动清理
- 不共享 fixture 写入（只读 fixture 用于集成测试和 snapshot）
- 测试间无依赖，可任意顺序执行

**被拒**：
- 共享测试 fixture 文件（多测试写同一文件会冲突）
- 运行时生成大图（P0 不需要，64×64 和 256×256 足够验证算法正确性）

---

## T4: Mock 策略

| 依赖 | 策略 | 理由 |
|------|------|------|
| 图片加载（image-rs） | 真实调用 | 纯本地库，无网络，加载快 |
| 文件系统 | 真实（tempfile） | Rust tempfile 标准做法，自动清理 |
| CLI 参数解析（clap） | 真实调用（assert_cmd） | 集成测试验证完整 CLI 路径 |
| 外部服务 | 不适用 | 无外部依赖 |

**结论**：无 Mock。全部真实调用。

**被拒**：
- Mock image-rs：增加测试复杂度，image-rs 是纯本地库不需要 Mock
- Mock 文件系统：tempfile 已是标准做法

---

## T5: CI 集成

> 引用 project.md CI 策略：test + clippy + fmt check。

| 项目 | 策略 |
|------|------|
| 测试运行 | `cargo test`（全量，含单元 + 集成 + snapshot） |
| Lint | `cargo clippy -- -D warnings`（0 warnings） |
| 格式 | `cargo fmt --check` |
| 失败策略 | 任一环节失败 → 阻断合并 |
| 超时 | 全量测试 < 2 分钟（纯像素计算，无 I/O 等待） |
| Snapshot 更新 | schema snapshot 变化时需人工 review（`cargo insta review` 或手动确认） |

**被拒**：
- CI 覆盖率报告（coverage）：P0 阶段不需要，手动验证 ≥80% 即可
- Nightly 大图测试：P0 阶段 fixture 最大 4000×4000，CI 已覆盖

---

## 测试规范

### 命名

```
// 单元测试：{功能}_{场景}
#[test]
fn zone_map_black_image() { ... }
#[test]
fn exposure_spot_missing_xy_returns_error() { ... }

// 集成测试：{命令}_{场景}
fn histogram_rgb_outputs_three_channels() { ... }
fn exposure_invalid_mode_returns_error() { ... }

// Schema snapshot：{命令}_schema
fn histogram_rgb_schema() { ... }
fn zone_map_schema() { ... }
```

### 执行顺序

- 单元测试：`cargo test --lib`（core）
- 集成测试：`cargo test --test integration_test`
- Schema snapshot：`cargo test --test schema_snapshot_test`
- 全量：`cargo test`（一次跑完）

### 验证命令

```bash
cargo test                    # 全量测试
cargo test --lib              # 仅 core 单元测试
cargo test --test integration_test  # 仅 CLI 集成测试
cargo test --test schema_snapshot_test  # 仅 schema snapshot
cargo clippy -- -D warnings   # Lint
cargo fmt --check             # 格式检查
```
