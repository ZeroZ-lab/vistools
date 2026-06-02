# Acceptance Log Template — 摄影计量 P0

> 每张真实照片复制一段，填完后即可统计 P0 是否通过。

## 汇总

| 指标 | 值 |
|------|----|
| 总样本数 |  |
| under 标签数 |  |
| correct 标签数 |  |
| over 标签数 |  |
| `exposure.assessment` 一致数 |  |
| 一致率 |  |
| `histogram --rgb` 提供增量信息数 |  |
| 增量占比 |  |
| `zone-map` 明显异常数 |  |
| 结论 | pass / partial / fail |

## 样本记录

### 文件：`photo-001.jpg`

- 场景类型：
- 人工标签：`under | correct | over`
- histogram：
  - 亮度结论：
  - RGB 增量信息：`yes | no`
  - 备注：
- zone-map：
  - 主分区：
  - 是否符合直觉：`yes | no`
  - 备注：
- exposure evaluative：
  - ev：
  - assessment：
- exposure highlight-weighted：
  - ev：
  - assessment：
- exposure spot：
  - 点位：
  - ev：
  - assessment：
- 最终判定：`pass | fail`
- 分歧说明：

### 文件：`photo-002.jpg`

- 场景类型：
- 人工标签：`under | correct | over`
- histogram：
  - 亮度结论：
  - RGB 增量信息：`yes | no`
  - 备注：
- zone-map：
  - 主分区：
  - 是否符合直觉：`yes | no`
  - 备注：
- exposure evaluative：
  - ev：
  - assessment：
- exposure highlight-weighted：
  - ev：
  - assessment：
- exposure spot：
  - 点位：
  - ev：
  - assessment：
- 最终判定：`pass | fail`
- 分歧说明：

## 统计规则

- 一致率 = `exposure.assessment` 与人工标签一致数 / 总样本数
- RGB 增量占比 = `histogram --rgb` 提供增量信息数 / 总样本数
- `zone-map` 明显异常数 > 0 时，先看是否为系统性偏差，再决定 `partial` 或 `fail`

## 决策门槛

- `pass`：
  - 一致率 ≥ 80%
  - RGB 增量占比 ≥ 30%
  - `zone-map` 无系统性异常
- `partial`：
  - 只有 1 项不达标，且可通过阈值或公式微调修复
- `fail`：
  - 多项不达标，或出现明显系统性错误
