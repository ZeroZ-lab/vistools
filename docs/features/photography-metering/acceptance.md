# Acceptance — 摄影计量 P0

> build 已完成后的手动验收阶段。目标不是再加命令，而是验证 P0 三个命令在真实照片上是否足够可信，能否支撑进入 P1。

## 版本信息

| 项目 | 值 |
|------|-----|
| 日期 | 2026-06-02 |
| 阶段 | 验收（L1 patch） |
| 输入 | P0 build + PRD 验收计划 + testing/contract.md |
| 输出 | 真实照片验收记录 + 是否进入 P1 的结论 |

配套文件：
- `acceptance-commands.md`：批量执行顺序与命令模板
- `acceptance-log-template.md`：逐张照片记录模板
- `acceptance-fixture-log.md`：固定 fixture 第一轮冒烟记录

---

## 验收目标

本阶段只验证 3 件事：

1. `histogram --rgb` 比原有亮度直方图更能支持曝光判断。
2. `zone-map` 的分区和代表区域在真实照片上语义稳定，不只是测试图上正确。
3. `exposure` 的 `ev` 与 `assessment` 在欠曝 / 正常 / 过曝照片上足够可信。

**不做**：
- 新命令开发（`focus-map` / `white-balance`）
- JSON schema 变更
- 算法扩写到 P1/P2
- 批量报告系统或自动化编排

---

## 输入数据

### 固定样本

- `fixtures/e2e/landscape_large.jpg`
- `fixtures/e2e/portrait_tall.jpg`
- `fixtures/e2e/panorama_wide.jpg`
- `fixtures/e2e/urban_square.jpg`
- `fixtures/e2e/screenshot_like.jpg`
- `fixtures/e2e/nature_small.jpg`
- `fixtures/e2e/nature_small.png`

### 待补充真实摄影样本

为避免只在通用 fixture 上自证正确，验收时还需要一组真实照片：

| 类别 | 最少数量 | 目的 |
|------|---------|------|
| 明显欠曝 | 5 | 验证 `exposure.assessment=under` |
| 正常曝光 | 5 | 验证 `assessment=correct` |
| 明显过曝 | 5 | 验证 `assessment=over` |
| 高对比逆光 | 5 | 验证 `highlight-weighted` 与 `evaluative` 差异 |
| 彩色光源 / 偏色场景 | 5 | 验证 `histogram --rgb` 的通道信息是否有额外价值 |

**最小通过样本量**：20 张。  
**推荐样本量**：25-50 张。

---

## 验收步骤

### A. 构建前置检查

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt -- --check
```

通过标准：
- 全量测试通过
- clippy 0 warnings
- fmt clean

### B. 真实图片命令检查

对每张验收图片至少执行：

```bash
cargo run -- histogram fixtures/e2e/landscape_large.jpg --rgb
cargo run -- zone-map fixtures/e2e/landscape_large.jpg
cargo run -- exposure fixtures/e2e/landscape_large.jpg --mode evaluative
cargo run -- exposure fixtures/e2e/landscape_large.jpg --mode highlight-weighted
```

抽样对局部区域再执行：

```bash
cargo run -- histogram fixtures/e2e/landscape_large.jpg --rgb --rect 0,0,800,600
cargo run -- zone-map fixtures/e2e/landscape_large.jpg --rect 0,0,800,600
cargo run -- exposure fixtures/e2e/landscape_large.jpg --mode spot --x 400 --y 300
```

### C. 人工判读记录

每张照片记录以下字段：

| 字段 | 说明 |
|------|------|
| 文件名 | 输入照片 |
| 人工标签 | under / correct / over |
| histogram 结论 | 是否从 RGB 通道得到额外判断信息 |
| zone-map 结论 | 主要像素是否落在预期分区 |
| exposure evaluative | ev + assessment |
| exposure highlight-weighted | ev + assessment |
| spot 采样点 | 如使用，记录点位与结果 |
| 是否通过 | pass / fail |
| 备注 | 分歧原因 |

---

## 通过标准

### P0 通过

满足以下条件即可认为摄影计量 P0 验收通过：

1. 真实照片样本中，`exposure.assessment` 与人工标签一致率 ≥ 80%。
2. `histogram --rgb` 至少在 30% 样本中提供了比单亮度 histogram 更强的信息。
3. `zone-map` 在抽查样本中没有出现明显违背直觉的主分区结果。
4. `histogram` 不带 `--rgb` 的输出形状保持兼容。
5. 没有新增会阻断 Agent 使用的错误码或参数歧义。

### P0 不通过

出现以下任一情况则不进入 P1：

- `assessment` 一致率 < 80%
- `zone-map` 在多张照片上出现系统性偏移
- `histogram --rgb` 在真实使用中几乎不提供增量价值
- spot / rect 模式在真实照片工作流里经常造成理解歧义

---

## 决策出口

| 验收结果 | 下一步 |
|---------|--------|
| 通过 | 进入 `摄影计量 P1 define`，范围限定为 `focus-map` / `white-balance` |
| 部分通过 | 先修 P0 算法或阈值，再复验；不新增命令 |
| 不通过 | 停止 P1 扩展，回到 idea/contract 重新收敛问题定义 |

---

## 验证记录模板

```md
### 文件：photo-001.jpg
- 人工标签：under
- histogram --rgb：蓝通道明显左偏，提供增量信息
- zone-map：Zone II-IV 占主导，符合预期
- exposure evaluative：ev=-0.82, assessment=under
- exposure highlight-weighted：ev=-0.34, assessment=correct
- spot：x=820,y=410 -> ev=-1.10, assessment=under
- 结论：pass
- 备注：高光区域较小，highlight-weighted 更乐观
```
