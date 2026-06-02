# v1 原子命令清单

> 目标：把 `vistools` 收束为 Agent 可编排的视觉测量基础设施，而不是通用图像处理库。

## 版本信息

| 项目 | 值 |
|------|-----|
| 日期 | 2026-06-02 |
| 阶段 | 轻量收敛（L1 patch） |
| 范围 | 命令面、协议边界、证据边界 |

---

## 目标与假设

**目标**：定义 v1 应该暴露哪些原子命令，哪些能力明确暂缓，确保后续实现沿着"视觉测量协议 + 证据链"推进。

**假设**：
- `vistools` 的主用户是会调用 CLI 的 Agent，而不是手动点按钮的人类用户。
- v1 要证明的是"Agent 能稳定测量并留下证据"，不是"覆盖所有视觉算法"。
- 编排层由 Agent 负责；`vistools` 负责提供稳定原子能力、结构化输出和坐标映射。

---

## 命令分层

v1 命令只分三层：

1. **视野层**：决定看哪里
2. **测量层**：决定量什么
3. **断言层**：决定是否符合预期

不在 v1 内加入行业命令或重型 workflow engine。

---

## v1 命令矩阵

| 命令 | 层级 | 状态 | 作用 | 为什么进 v1 |
|------|------|------|------|-------------|
| `inspect` | 视野层 | 已有 / 保留 | 读元数据并给出下一步建议 | Agent 进入任何图像任务的起点 |
| `overview` | 视野层 | 已有 / 保留 | 生成全局缩略图并提供缩放映射 | 支撑 overview → zoom-in 工作流 |
| `tile` | 视野层 | 已有 / 保留 | 把大图拆成稳定网格 | 支撑全覆盖检查和多块遍历 |
| `viewport` | 视野层 | 已有 / 保留 | 以 anchor/percent/rect 精确裁剪局部 | 支撑局部放大和坐标回映 |
| `sample` | 测量层 | 已有 / 保留 | 点取色或区域平均取色 | 第一批真实测量原语，已验证 JSON 形状 |
| `diff` | 测量层 | 已实现 | 比较两张图或两个区域的像素差异 | 设计还原、回归检查、变化检测的基础原语 |
| `measure` | 测量层 | v1 新增 | 输出区域尺寸、边界框、距离等基础几何量 | 把"看起来差不多"变成可验证数字 |
| `assert-color` | 断言层 | v1 新增 | 对颜色测量结果做容差判断 | 把取色结果转成 Agent 可执行结论 |
| `assert-diff` | 断言层 | v1 新增 | 对 diff 结果做阈值判断 | 让视觉比较能直接产出 pass/fail |
| `assert-region` | 断言层 | v1 新增 | 对区域尺寸/位置/重叠关系做判断 | 支撑 spacing/alignment 一类基础检查 |

---

## 明确不进 v1

| 类型 | 不做项 | 原因 |
|------|--------|------|
| 行业命令 | `check-ecommerce-image` / `check-industrial-defect` / `check-design-page` | 会把平台层拉成场景方案层 |
| 通用像素处理 | `resize` / `rotate` / filter / enhance | 偏离"Agent 视觉测量"主线，且已有成熟替代 |
| 智能识别 | 目标检测 / 主体分割 / OCR / 版面理解 | 会直接进入 VLM/CV 主战场，范围失控 |
| 工作流接管 | 内建 orchestrator / DSL / 自动计划执行器 | 当前阶段让 Agent 做编排更轻、更灵活 |
| 解释输出 | 自然语言报告生成 | 应由 LLM 基于结构化结果完成，不由工具生成 |

---

## 最小闭环

v1 至少要能支持下面这条可复用链路：

```text
inspect
→ overview 或 tile
→ viewport
→ sample / diff / measure
→ assert-color / assert-diff / assert-region
→ evidence artifacts + structured JSON
```

如果某个新增命令不能进入这条链路，就不应优先进入 v1。

---

## 统一输出要求

所有 v1 命令都应保持以下约束：

- 返回统一 `CommandResult<T>` 信封
- 错误使用稳定 `error.code`
- 测量结果必须能映射回源图坐标
- 涉及输出图片的命令必须保留 `coordinate_mapping`
- 断言命令必须同时返回 `measured`、`expected`、`tolerance`、`pass`
- 可审计命令必须允许挂接 evidence 路径或 region 引用

这意味着 v1 不是"一组命令"，而是一种统一事实格式。

---

## 实现优先级

### P0：已存在并继续稳定

- `inspect`
- `overview`
- `tile`
- `viewport`
- `sample`
- `diff`

### P1：v1 必补

- `measure`
- `assert-color`
- `assert-diff`
- `assert-region`

### P2：等 v1 跑通后再评估

- `grid`
- `zoom`
- `lens`

这些命令不是没价值，而是当前不如 `diff / measure / assert-*` 更接近"测量 + 验证 + 证据"闭环。

---

## 成功标准

满足以下条件，说明 v1 命令面收敛是对的：

1. Agent 能稳定跑通至少一条视觉测量闭环，而不是只停在裁剪图片。
2. 至少两类任务可复用同一套原子命令，例如颜色检查和设计还原检查。
3. 输出结果可被 LLM 解释，但无需 LLM 参与即可审计。
4. 新需求首先映射到原子能力，不需要新增业务命令。

---

## 一句话定位

`vistools` v1 = **给 Agent 使用的视觉测量命令面：视野控制 + 原子测量 + 结构化断言 + 坐标化证据。**
