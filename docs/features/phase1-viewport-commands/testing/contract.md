# Testing Contract — Phase 1: 视野控制命令集

> T1-T5 测试策略决策

## T1: 测试类型

| 类型 | 占比 | 目标 | 选择理由 |
|------|------|------|---------|
| 单元测试 | 70% | core 库每个模块的纯函数逻辑 | Rust 单元测试零成本（编译器内置），`cargo test` 毫秒级。coord/guard 是纯函数，天然适合单测。 |
| 集成测试 | 25% | CLI 端到端：参数解析 → JSON 输出 schema | assert_cmd 验证 CLI 实际执行 + 输出格式。捕获参数解析错误、JSON 序列化问题、文件 I/O 问题。 |
| Schema 快照 | 5% | JSON 输出 schema 不变 | 每个命令一次成功执行，把 JSON 输出存为 `.snap` 文件。后续改动如果改了 schema，测试直接失败。 |

**被拒**：
- 属性测试（proptest）：对坐标计算有价值但 Phase 1 复杂度不够高，投入产出比低。
- 基准测试（criterion）：Phase 1 不做性能回归，只在手动验收时检查耗时。
- 100% 行覆盖：虚荣指标。guard/coord 等关键路径全覆盖，其他模块覆盖正常+错误路径即可。

---

## T2: 覆盖策略

| 模块 | 可自动化 | 测试类型 | 覆盖程度 | 理由 |
|------|---------|---------|---------|------|
| types | 是 | 单元 | 基础 | 数据结构，序列化/反序列化测试即可 |
| guard | 是 | 单元 + 集成 | **全覆盖** | 安全边界，每个校验规则必须测试 |
| coord | 是 | 单元 | **全覆盖** | 坐标计算是核心逻辑，anchor 9 个方位 + percent + mapping 全覆盖 |
| inspect | 是 | 单元 + 集成 | **全覆盖** | P0 命令，3 个验收条件全测 |
| overview | 是 | 单元 + 集成 | **全覆盖** | P0 命令，缩放+映射+边界全测 |
| tile | 是 | 单元 + 集成 | **全覆盖** | P0 命令，余数策略是关键测试点 |
| viewport | 是 | 单元 + 集成 | **全覆盖** | P0 核心命令，三种模式 + 越界 + 零面积 |
| resize | 是 | 单元 + 集成 | 正常+错误 | P1 命令，等比例+强制+缺参数 |
| rotate | 是 | 单元 + 集成 | 正常+错误 | P1 命令，90/180/270/0/45° |
| main (CLI) | 是 | 集成 | 冒烟测试 | 每个命令至少一次 CLI 调用 + JSON 验证 |

**关键路径全覆盖**：guard → coord → inspect → overview → tile → viewport

**被拒**：
- 全模块 100% 行覆盖：resize/rotate 是 P1，正常+错误路径足够。
- 不测 types：序列化是基础设施，不测会漏 Serde derive 错误。

---

## T3: 测试数据

| 数据类型 | 用途 | 构造方式 |
|---------|------|---------|
| 64x64 纯色 PNG | guard/types 单元测试 | Rust 代码生成（image-rs 创建 → 保存到 fixtures/） |
| 256x256 渐变 PNG | inspect/overview 单元测试 | Rust 代码生成 |
| 1000x1000 棋盘格 PNG | tile/viewport 单元测试 | Rust 代码生成（棋盘格便于验证裁剪位置） |
| 6000x4000 截图模拟 PNG | 集成测试（不放入 git，CI 时生成） | 测试 helper 函数动态生成 |
| 不存在的文件路径 | 错误场景 | 用 `"/tmp/no-such-file.png"` |
| 超像素限制图片（10001x10001） | guard 边界 | 不实际创建，只 mock Size |

**构造方式**：
- 小 fixture（≤1MB）提交到 `fixtures/` 目录（git tracked）
- 大 fixture（6000x4000）CI 时动态生成，不提交
- 每个 fixture 一次生成，所有测试复用

**被拒**：
- 真实截图作为 fixture：文件大、格式不可控、不可重复
- 全部动态生成：每次 `cargo test` 都要等生成，小 fixture 直接提交更快

---

## T4: Mock 策略

| 依赖 | 策略 | 理由 |
|------|------|------|
| image-rs | 真实调用 | 纯本地库，稳定，无网络。Mock 它会掩盖格式编解码 bug。 |
| 文件系统 | 真实调用 + tempdir | 使用 `tempfile::tempdir()` 创建临时目录，测试结束自动清理。 |
| stdout | 真实写入 + assert_cmd 捕获 | assert_cmd 自动捕获 CLI 的 stdout/stderr。 |
| 网络请求 | 无（Phase 1 没有） | — |

**隔离规则**：
- 每个测试创建自己的 tempdir，互不干扰
- 不使用共享全局 fixture 文件（只读共享可以）
- 测试顺序无关（`cargo test -- --test-threads=4` 可并行）

**被拒**：
- Mock image-rs：掩盖真实编解码 bug，测试变成测 mock 而非测逻辑
- 全局共享 tempdir：测试间互相干扰，并行会失败

---

## T5: CI 集成

| 项目 | 策略 |
|------|------|
| 测试命令 | `cargo test --workspace` |
| Lint | `cargo clippy --workspace -- -D warnings` |
| 格式 | `cargo fmt --check --all` |
| 测试超时 | 5 分钟（纯本地计算，无网络，应该 < 30s） |
| 失败策略 | 测试失败 → 阻断合并 |
| 覆盖率 | Phase 1 不强制覆盖率门槛，`cargo tarpaulin` 输出报告供参考 |
| 平台 | GitHub Actions: macOS (ARM) + Linux (x64) |
| 二进制大小检查 | `cargo build --release` 后 `ls -la` 检查 ≤ 8MB |

**CI 流程**：
```
on: push, PR
jobs:
  test:
    - cargo fmt --check
    - cargo clippy -- -D warnings
    - cargo test --workspace
    - cargo build --release
    - check binary size ≤ 8MB
```

**被拒**：
- 覆盖率门槛（如 80%）：Phase 1 先验证功能正确，覆盖率门槛留 Phase 2
- Windows CI：Phase 1 先支持 macOS + Linux，Windows 后续补
- tarpaulin 强制失败：覆盖率是参考指标，不阻断

---

## 测试规范

### 命名

```rust
// 单元测试：模块内 #[cfg(test)] mod tests
// 测试函数命名：test_<功能>_<场景>_<预期>
#[test]
fn test_anchor_right_on_6000x4000_returns_right_half() {}

#[test]
fn test_tile_4x3_on_6000x4000_produces_12_tiles() {}

#[test]
fn test_viewport_rect_out_of_bounds_returns_error() {}
```

### 集成测试

```rust
// tests/<command>_test.rs
// 用 assert_cmd 调用 CLI 二进制
use assert_cmd::Command;

#[test]
fn inspect_returns_json_with_dimensions() {
    let cmd = Command::cargo_bin("image-viewport")
        .unwrap()
        .args(["inspect", "fixtures/256x256.png", "--json"])
        .assert();
    cmd.success();
    let output: serde_json::Value = /* parse stdout json */;
    assert_eq!(output["source"]["width"], 256);
    assert_eq!(output["source"]["height"], 256);
    assert_eq!(output["ok"], true);
}
```

### 隔离

- 每个集成测试用 `tempfile::tempdir()` 创建输出目录
- 不写入 fixtures/ 目录
- 不依赖其他测试的输出
- 可 `--test-threads=N` 并行

### 测试文件结构

```
tests/
├── guard_test.rs         # guard 模块集成测试
├── coord_test.rs         # coord 模块集成测试（通过 CLI）
├── inspect_test.rs       # inspect 命令 E2E
├── overview_test.rs      # overview 命令 E2E
├── tile_test.rs          # tile 命令 E2E
├── viewport_test.rs      # viewport 命令 E2E
├── resize_test.rs        # resize 命令 E2E
├── rotate_test.rs        # rotate 命令 E2E
└── integration_test.rs   # 全量冒烟测试（所有命令串联）

fixtures/
├── 64x64.png
├── 256x256.png
└── 1000x1000.png
```
