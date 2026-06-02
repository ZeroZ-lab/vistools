# Acceptance Fixture Log — 摄影计量 P0

> 这是一轮固定 fixture 的初始冒烟记录。它只能证明命令在现有真实尺寸图片上输出稳定，不能替代真实摄影样本验收。

## 前置验证

- `cargo test`：通过，134 tests passed
- `cargo clippy -- -D warnings`：通过
- `cargo fmt -- --check`：通过

执行备注：
- 当前环境下直接 `cargo ...` 会被代理层误解析。
- 实际执行使用了工具链 bin 目录优先的 PATH：
  - `env PATH=/Users/zhengjianqiao/.rustup/toolchains/1.88.0-aarch64-apple-darwin/bin:$PATH cargo ...`

## 发现的契约偏差

- `histogram` / `zone-map` / `exposure` 默认输出 JSON，不接受 `--json`
- 之前验收文档里的 `--json` 命令示例已修正

## 样本记录

### 文件：`fixtures/e2e/landscape_large.jpg`

- 场景类型：大尺寸风景图
- histogram：
  - `mean_luma=117.19`
  - `median_luma=88`
  - `p05=12`, `p95=238`
  - RGB 增量信息：`yes`
  - 备注：`R/G/B` 三通道中蓝通道 `p50=65`，明显低于红绿通道，说明通道分布并不对称；仅看亮度直方图会丢失这层信息
- zone-map：
  - 主分区：`Zone I (16.93%)`、`Zone 0 (16.03%)`、`Zone X (15.72%)`、`Zone IX (14.14%)`
  - 是否符合直觉：`yes`
  - 备注：整体呈强对比分布，暗部和高光两端占比都高，符合风景图高动态范围直觉
- exposure evaluative：
  - `ev=-0.0103`
  - `assessment=correct`
- exposure highlight-weighted：
  - `ev=1.0175`
  - `assessment=over`
- 最终判定：`pass`
- 分歧说明：`highlight-weighted` 明显更敏感，说明高光占比足以改变总体判断，这正是该模式存在的价值

### 文件：`fixtures/e2e/portrait_tall.jpg`

- 场景类型：竖幅人像/建筑风格长图
- histogram：
  - `mean_luma=138.70`
  - `median_luma=141`
  - `p05=58`, `p95=219`
  - RGB 增量信息：`yes`
  - 备注：红通道 `mean=143.00` 明显高于蓝通道 `mean=125.92`，表明画面偏暖；亮度直方图单独无法表达这点
- exposure evaluative：
  - `ev=0.2333`
  - `assessment=correct`
- 最终判定：`pass`
- 分歧说明：暂无；整体亮度集中在中高区域，但未跨过 `over` 阈值

### 文件：`fixtures/e2e/screenshot_like.jpg` 区域 `--rect 0,0,800,600`

- 场景类型：截图风格局部区域
- zone-map：
  - 主分区：`Zone II (22.27%)`、`Zone IX (22.22%)`、`Zone VIII (14.20%)`
  - 是否符合直觉：`yes`
  - 备注：左上局部既有深色 UI 区块也有亮背景区域，双峰分布合理

### 文件：`fixtures/e2e/screenshot_like.jpg` 点测光 `--mode spot --x 400 --y 300`

- 场景类型：截图风格单点
- exposure spot：
  - `ev=0.8164`
  - `assessment=over`
  - `spot_point=(400,300)`
- 最终判定：`pass`
- 分歧说明：这是局部点测光结果，不等价于整图曝光判断

### 文件：`fixtures/e2e/urban_square.jpg`

- 场景类型：方形城市图
- exposure center-weighted：
  - `ev=0.4208`
  - `assessment=correct`
- 最终判定：`pass`
- 分歧说明：接近 `over` 边界但仍在 `correct` 区间内，适合作为后续真实照片验收时的边界样本

### 文件：`fixtures/e2e/landscape_large.jpg`（兼容性检查）

- histogram 不带 `--rgb`：
  - 输出中无 `rgb` 字段
  - 与向后兼容约束一致
- 最终判定：`pass`

## 当前结论

- 固定 fixture 冒烟通过，未见明显结构错误或违背直觉的输出
- `histogram --rgb` 的增量信息在至少 2 个样本上已经可见
- `highlight-weighted` 与 `evaluative` 的分歧是可解释的，不像算法抖动
- 还不能据此宣布 P0 验收通过，因为缺少“真实摄影样本 + 人工曝光标签”的统计

## 下一步

1. 准备 20-50 张真实摄影样本，按 `under / correct / over` 先做人标
2. 复用 [acceptance-commands.md](/Users/zhengjianqiao/workspace/vistools/docs/features/photography-metering/acceptance-commands.md:1) 跑命令
3. 把结果填入 [acceptance-log-template.md](/Users/zhengjianqiao/workspace/vistools/docs/features/photography-metering/acceptance-log-template.md:1)
4. 统计一致率、RGB 增量占比、zone-map 异常数，再决定是否进入 P1
