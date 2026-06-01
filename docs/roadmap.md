# Roadmap — vistools

> vistools 是 LLM 的视觉仪器层：看全局、看局部、放大、取色、测量、对比，并且所有输出都能映射回源图坐标。

## 产品边界

### 做什么

| 能力 | 人类类比 | 目标 |
|------|----------|------|
| inspect / overview | 人眼 + 望远镜 | 先判断图有多大，再用模型可读尺寸看全局 |
| viewport / tile | 注视 + 扫描 | 聚焦指定区域，或把未知大图分块检查 |
| zoom | 显微镜 | 把局部细节放大给 LLM 看，同时保留坐标映射 |
| sample | 取色器 | 读取点/区域颜色、alpha 和区域平均色 |
| grid | 坐标纸 | 给图加网格/坐标标记，让 LLM 更容易指代位置 |
| diff | 变化感知 | 对比 before/after，输出变化区域和源图坐标 |
| lens | 视觉镜片 | 灰度、反色、alpha、边缘、对比增强，用于暴露低可见度问题 |
| measure | 尺子 | 测量距离、尺寸、间距和区域统计 |

### 不做什么

| 不做 | 理由 |
|------|------|
| 通用 resize / rotate / convert | 属于图片处理库能力，不是视野导航核心 |
| 修图滤镜 / 美化 | 不服务 Agent 判断和定位 |
| OCR / 语义理解 | 需要模型能力，当前保持本地确定性 CLI |
| Web UI / 登录 / 云端处理 | Phase 1-3 聚焦本地 Agent 工作流 |

## 阶段路线

### v0.2 — 视野核心稳定

**目标**：让当前 4 个公开命令成为可靠的 LLM 眼睛。

| 命令 | 状态 | 重点 |
|------|------|------|
| inspect | 已有 | 输出更直接的 `recommended_next` 和原因 |
| overview | 已有 | 使用 `--max-side`，保证长边适配视觉模型阈值 |
| viewport | 已有 | 严格校验 percent 越界，避免隐式 clamp |
| tile | 已有 | 保持完整覆盖和稳定 source_region |

**验收信号**：
- `cargo test` / `cargo clippy -- -D warnings` / `cargo fmt --check` 全部通过
- release 二进制 ≤ 8MB
- README 和 Skill 只主推 inspect / overview / viewport / tile
- 真实截图工作流可完成：inspect → overview → viewport → 源图坐标定位
- CLI JSON schema shape 快照覆盖 4 个核心命令和错误信封

### v0.3 — 第一批视觉仪器

**目标**：补齐“显微镜、取色器、坐标纸”，让 LLM 能看细节、读颜色、精确指代区域。

| 命令 | 优先级 | 输出 |
|------|--------|------|
| sample | 已实现 | 点/区域颜色，包含 RGB、HEX、alpha、区域平均色和透明度统计 |
| zoom | P0 | 裁剪局部并放大，包含从放大图到源图的坐标映射 |
| grid | P1 | 叠加坐标网格/标签，方便 LLM 说“看 B4 区域” |

**验收信号**：
- sample 点取色和区域统计有单元测试，覆盖透明像素、CLI 集成测试和 schema 快照
- zoom 支持 nearest / high-quality 两种放大策略
- grid 输出不破坏源文件，网格间距、标签可预测
- 三个命令都返回统一 `CommandResult<T>`

### v0.4 — 视觉验证闭环

**目标**：从“看图”升级到“验证变化”。

| 命令 | 优先级 | 输出 |
|------|--------|------|
| diff | P0 | before/after 差异区域、变化比例、建议 viewport |
| lens | P1 | grayscale / invert / alpha / edges / contrast 等视觉增强模式 |
| measure | P1 | 两点距离、矩形尺寸、区域边界和间距统计 |

**验收信号**：
- diff 输出 changed_regions，包含 rect、change_score、source_region
- diff 能忽略微小抗锯齿噪声或提供阈值参数
- lens 是视觉增强，不提供通用滤镜堆叠
- measure 输出适合 Agent 直接判断 UI 间距是否偏差

### v0.5 — Agent 工作流封装

**目标**：把单命令组合成更少决策成本的 Agent 工作流。

| 能力 | 形式 | 目标 |
|------|------|------|
| workflow hints | inspect 输出 | 根据尺寸/比例建议 overview、tile 或 viewport |
| drilldown | 命令或 Skill 流程 | 串联 overview → viewport → zoom |
| visual QA recipe | Skill 文档 | 前端修改后固定执行截图检查流程 |

**验收信号**：
- Skill 明确规定未知图片先 inspect
- 大图检查流程不超过 3 次命令调用即可定位局部区域
- Agent 报告必须包含源图坐标和使用过的命令链

## 推荐实现顺序

1. `sample`：已实现第一版点取色、区域平均色、alpha 统计。
2. 实现 `zoom`：显微镜能力，复用 viewport + scale mapping。
3. 实现 `grid`：提高 LLM 指代位置的可靠性。
4. 实现 `diff`：进入 before/after 视觉验证闭环。
5. 再考虑 `lens` 和 `measure`：它们有用，但需要更清晰的模式边界。

## 暂缓项

| 项 | 暂缓理由 |
|----|----------|
| screenshot 命令 | Browser/Playwright 已有成熟能力，vistools 先处理图片文件 |
| OCR | 本地 OCR 会增加依赖和体积，且与视觉导航边界不同 |
| MCP server | CLI JSON 已满足 Agent 调用，先不增加维护面 |
| 远端 AI 分析 | 涉及账号、计费、隐私和服务端，不影响本地仪器层验证 |
