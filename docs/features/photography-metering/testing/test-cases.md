# Test Cases — 摄影计量 P0

> 从 PRD AC + contract.md 验证标准推导。完整测试策略由 test 阶段补充。

## 测试范围矩阵

| 命令 | 单元测试 | 集成测试 | Schema Snapshot | PRD AC 覆盖 |
|------|---------|---------|-----------------|------------|
| histogram --rgb | ✅ 新增 2 | ✅ 新增 2 | ✅ 1（带 --rgb）+ 验证现有不变 | AC-01-1~4 |
| zone-map | ✅ 新增 4 | ✅ 新增 1 | ✅ 1 | AC-02-1~5 |
| exposure | ✅ 新增 7 | ✅ 新增 2 | ✅ 1 | AC-03-1~10 |

## P0 用例骨架

### histogram --rgb（Task-01）

| 用例 | 类型 | 输入 | 预期 |
|------|------|------|------|
| RGB 三通道输出 | 单元 | RGB 渐变图 + rgb=true | r/g/b 各 256 bins + 分位数 + clipping |
| 向后兼容 | 单元 | 任意图 + rgb=false | HistogramMetrics 无 rgb 字段 |
| 通道 clipping | 单元 | 红过曝图 + rgb=true | r.clipping_high > 0, g/b ≈ 0 |
| rect 区域 RGB | 集成 | --rect 100,100,200,200 --rgb | RGB 统计仅覆盖 rect |
| schema 锁定 | snapshot | --rgb | JSON 形状锁定 |

### zone-map（Task-02）

| 用例 | 类型 | 输入 | 预期 |
|------|------|------|------|
| 纯黑图 | 单元 | 64x64 全黑 | Zone 0 ratio=1.0, 其余 0 |
| 纯白图 | 单元 | 64x64 全白 | Zone X ratio=1.0, 其余 0 |
| 渐变图 | 单元 | 渐变 PNG | 11 zones 均有分布, ratio 之和 ≈ 1.0 |
| representative_rect | 单元 | 渐变图 | 每个 zone 的 rect 在源图范围内 + 包含该 zone 像素 |
| rect 区域 | 集成 | --rect 0,0,500,500 | 统计仅覆盖 rect |
| schema 锁定 | snapshot | 默认 | JSON 形状锁定 |

### exposure（Task-03）

| 用例 | 类型 | 输入 | 预期 |
|------|------|------|------|
| 中灰图 ev≈0 | 单元 | luma=118 | ev ≈ 0.0, assessment="correct" |
| 过曝图 | 单元 | luma=220 | ev > 1.0, assessment="over" |
| 欠曝图 | 单元 | luma=30 | ev < -1.0, assessment="under" |
| spot 模式 | 单元 | --mode spot --x 10 --y 10 | ev 基于单像素 luma |
| spot 缺参数 | 单元 | --mode spot（无 --x/--y） | error.code = INVALID_PARAMETERS |
| 无效模式 | 单元 | --mode invalid | error.code = INVALID_PARAMETERS |
| assessment 边界 | 单元 | luma 精确控制 | ev=-0.5→correct, ev=0.5→correct, ev=-0.6→under |
| center-weighted | 单元 | 中心亮四周暗 | ev 偏向中心（比 evaluative 更高） |
| highlight-weighted | 单元 | 含高光区域 | ev 基于 top 10% luma |
| rect 区域 | 集成 | --rect 200,200,400,400 | 统计仅覆盖 rect |
| schema 锁定 | snapshot | --mode evaluative | JSON 形状锁定 |
