# Idea Brief — Agent Image Viewport CLI

> 探索阶段的产出：方向地图 + 判断标准 + MVP 定义

## 版本信息

| 项目 | 值 |
|------|-----|
| 日期 | 2026-06-01 |
| 参与者 | ZeroZ-lab |
| 探索轮次 | 4（第1轮方向分析→第2轮市场竞品+场景验证→第3轮竞品深挖+技术+痛点+产品形态→第4轮架构决策：CLI-only + 登录制解锁远端 AI） |

---

## 痛点

**一句话描述**：AI Agent 面对大图（截图、设计稿、长网页、图表、扫描件）时，缺少一个可编程的视觉视野控制层，导致成本高、细节丢失、无法局部探索。

**具体场景**：AI Coding Agent 修改了 React 组件的 CSS，Playwright 截了一张 6000×4000 的全页截图。Agent 把整张截图发给视觉模型，模型说"看起来没问题"。但实际上右侧边栏的按钮文字溢出了——因为整张图缩放后，局部细节已经模糊到不可读。

**谁受影响**：
- AI Coding 工具用户（Claude Code / Cursor / Codex 用户）
- 前端开发团队（视觉回归 + Agent 修复闭环）
- 设计系统团队（设计还原和 UI 一致性）
- Computer-use Agent 开发者（截图预处理和隐私清洗）

**影响程度**：高频 × 高严重度。Agent 每次改前端代码都需要视觉验证，每次大图输入都有成本和精度问题。

**当前方案 + 不足**：
- 直接把整张图发给视觉模型 → 成本高、细节丢失、模型注意力被稀释
- 用 ImageMagick 手动裁剪 → 语义缺失、无 JSON schema、无坐标映射、不是 Agent-native
- **imgctl (agent-rt/imgctl)** → 最接近的竞品：Rust CLI、16个命令、Agent-first。但缺少 overview 语义、viewport 导航抽象、递归视野探索、坐标映射随输出、MCP 包装
- Playwright visual comparison → 偏断言工具，不是 Agent 的视觉探索工具
- Cloudinary MCP → 云端重资产，不是本地轻量 CLI
- Peekaboo (~4,600 stars) / gsd-browser (~235 stars) → 做截图捕获和浏览器自动化，不做通用图片视野导航

**根因分析**（5 Whys）：
1. Agent 为什么看不准 UI 细节？→ 整张大图缩放后局部分辨率不够
2. 为什么不裁剪局部再看？→ 缺少一个 Agent 可调用的、语义化的裁剪工具
3. 为什么 ImageMagick 不够？→ 它是给人用的瑞士军刀，不是 Agent-native 工具（无 JSON schema、无坐标映射、无安全边界）
4. 为什么没有 Agent-native 的图片视野工具？→ Agent 工具生态刚起步，MCP 标准才成型
5. 为什么现在是机会窗口？→ computer-use / AI Coding / MCP 三重趋势叠加，Agent 对视觉输入的控制需求正在爆发

---

## 方向地图

### 方向对比

| 方向 | 核心思路 | 适用场景 | 风险 | 验证难度 |
|------|---------|---------|------|---------|
| A: AI Coding Screenshot Inspector | Agent 改前端代码后，用 CLI 切分/裁剪/检查页面截图，定位 UI 问题 | AI Coding 视觉验收闭环 | 需绑定 Playwright 截图流程 | 中 |
| B: Design-to-Code 局部视觉理解 | Agent 收到设计稿截图，分区裁剪后局部生成代码 | 非结构化设计图转代码 | Figma MCP 会吸走部分价值 | 中高 |
| C: 大图通用视野控制 CLI | 不绑定特定场景，做通用的 inspect/tile/crop/zoom 工具 | 所有 Agent 面对大图的场景 | 过于通用可能缺乏杀手场景 | 低 |

### 方向详情

**方向 A: AI Coding Screenshot Inspector**
- **做什么**：Agent 修改前端代码 → Playwright 截图 → CLI 切分/裁剪 → Agent 看局部 → 定位问题 → 继续改代码
- **适用场景**：AI Coding 工具需要视觉验收闭环时，这是最直接的解法
- **优势**：痛点最强、用户最明确、Agent 使用频率高、MCP 化自然、可与 React/Playwright/Cursor/Claude Code 结合
- **劣势**：需要与 Playwright 截图流程配合，但 Playwright 已是 AI Coding 标准工具
- **参考产品**：Playwright visual comparison（做断言）、Percy/Chromatic（做 CI 截图对比）
- **研究发现**：Claude 文档建议"pre-resizing, cropping, or both"；Playwright 已内置截图对比；OpenAI/Anthropic computer use 都依赖截图感知
- **评分**：痛点 9.5 / Agent 适配 9.8 / 商业价值 8.5 / MVP 难度 6.5 → **综合 9.4**

**方向 B: Design-to-Code 局部视觉理解**
- **做什么**：设计稿截图 → overview 看整体 → tile 分块 → crop 各区域 → Agent 分区生成组件
- **适用场景**：非结构化设计图（截图、竞品页面、旧设计稿）转代码，Figma 数据不可用时
- **优势**：提升还原准确度、支持组件化生成、兼容非 Figma 输入
- **劣势**：Figma MCP 提供结构化数据，比截图更强；此方向不会独立消失但会被部分替代
- **参考产品**：Figma MCP（结构化设计上下文）、v0.dev（设计图转代码）
- **研究发现**：Figma MCP 已在做设计上下文 → AI Agent → 代码生成，但大量真实输入不是 Figma 文件
- **评分**：痛点 9.0 / Agent 适配 9.5 / 商业价值 8.5 / MVP 难度 7.0 → **综合 9.0**

**方向 C: 大图通用视野控制 CLI**
- **做什么**：通用 inspect/overview/tile/viewport/resize/rotate，不绑定特定场景
- **适用场景**：所有 Agent 面对大图需要视野控制的场景——长截图阅读、图表分析、文档扫描、computer-use 预处理等
- **优势**：最灵活、MVP 最简单（8 个命令）、可作为基础设施组件
- **劣势**：缺乏单一杀手场景，容易变成"又一个 ImageMagick wrapper"
- **参考产品**：ImageMagick、Sharp、Squoosh、libvips（通用图片处理）
- **研究发现**：现有工具缺少 Agent-native 特性（JSON schema、坐标映射、安全边界、MCP tool schema）
- **评分**：痛点 8.0 / Agent 适配 9.0 / 商业价值 7.5 / MVP 难度 5.0 → **综合 8.2**

### 其他已评估方向（记录但不展开）

| 方向 | 综合评分 | 一句话理由 |
|------|---------|----------|
| 长网页/长截图阅读 | 8.8 | MVP 最容易，可做方向 A 的附带场景 |
| Computer-use 视觉预处理 | 8.7 | 技术价值高但产品化要绑定具体 Agent 宿主 |
| 文档/扫描件阅读 | 8.5 | 容易扩展到 OCR 大系统，第一版不要碰 |
| 隐私打码/安全预处理 | 8.3 | 适合做安全特性而非独立产品 |
| 图表/Dashboard 分析 | 8.2 | 偏窄，可做后续场景 |
| 内容/电商图片资产处理 | 7.9 | 竞争多，容易变普通工具 |
| 普通 resize/convert CLI | 6.3 | 伪需求——不是 Agent-native |
| 滤镜/美化工具 | 5.0 | 伪需求——与 Agent 工作流无关 |

---

## 推荐方向 + 评估

**选择**：方向 A（AI Coding Screenshot Inspector）为主，方向 C（大图通用视野控制 CLI）为基底

**理由**：
1. 方向 A 痛点最强（9.5）且 Agent 适配度最高（9.8），是最清晰的杀手场景
2. 方向 C 是方向 A 的技术基底——先做通用 CLI，再用 AI Coding 场景验证价值
3. 两者不矛盾：CLI 是底层能力，AI Coding Inspector 是上层场景

**策略**：以"Agent Image Viewport"为产品定位（方向 C 的广度），以"AI Coding Screenshot Inspector"为首发场景（方向 A 的深度）

**被拒方向及理由**：
- 普通图片处理 CLI / 滤镜美化：与 Agent 工作流无关，是伪需求
- OCR/版面分析大系统：第一版范围失控风险高
- 独立隐私打码产品：更适合做特性而非产品

**评估矩阵**：

| 标准 | 权重 | 方向 A (AI Coding Inspector) | 方向 B (Design-to-Code) | 方向 C (通用视野 CLI) |
|------|------|------------------------------|-------------------------|---------------------|
| 痛点解决度 | 高 | 9.5 | 9.0 | 8.0 |
| 验证难度 | 高 | 6.5 (中) | 7.0 (中高) | 5.0 (低) |
| 团队能力匹配 | 中 | 9.0 (Rust + 前端) | 8.5 | 9.0 |
| 市场规模 | 中 | 8.5 (AI Coding 用户快速增长) | 8.5 | 7.5 |
| 时间窗口 | 低 | 9.0 (MCP 刚成型，先发优势) | 8.5 | 8.0 |
| **加权总分** | | **8.5** | **8.1** | **7.5** |

---

## MVP 定义

> ⚠️ 已在第4轮更新，详见底部「最终 MVP 定义（第4轮更新）」

**核心功能**（去掉任何一个就不成立）：
1. `inspect` — 获取图片 metadata（宽高、格式、文件大小），Agent 据此决定下一步操作
2. `overview` — 缩放到指定最大宽度，生成总览图，Agent 用于"扫一眼全局"
3. `tile` — 按 rows×cols 分块切割，Agent 用于"逐块检查"
4. `viewport` — 按 anchor / percent / rect 裁剪局部，Agent 用于"放大看细节"
5. JSON-first 输出 + 坐标映射 — 所有操作返回结构化 JSON，包含源图坐标映射

**边界声明**（明确不做）：
- 不做 OCR / 版面分析 / 表格识别，因为第一版核心验证是"Agent 是否会调用这个工具控制视觉输入"
- 不做 AI 自动主体检测 / 智能裁剪，因为增加复杂度且不确定 Agent 是否需要
- 不做 MCP server — CLI-only 架构，Agent 通过 Bash 调用
- 不做 IDE 插件 / UI，因为第一版验证 CLI 足够
- 不做 visual diff / compare，这是第二阶段功能

**验证标准**（怎么算 MVP 成功）：
- Agent 能完成：截图 → inspect → overview → tile → viewport → 定位局部问题的完整闭环
- 所有命令输出稳定 JSON schema，包含坐标映射
- Agent 在 AI Coding 场景中，使用此工具后的视觉识别准确率 > 不使用此工具
- 3 个早期用户反馈"有用"（不是"花哨"）

---

## 假设清单

> ⚠️ 已在第4轮更新，详见底部「最终假设清单（第4轮更新）」

| # | 假设 | 验证方法 | 成功标准 | 负责人 | 时间 |
|---|------|---------|---------|--------|------|
| H1 | AI Coding Agent 会通过 Bash 主动调用 CLI 控制视觉输入 | 构建 CLI，让 Claude Code 在真实前端任务中使用 | Agent 在 ≥50% 的前端修改任务中主动调用 inspect/tile/viewport | ZeroZ-lab | 2 周 |
| H2 | 局部裁剪后模型视觉识别准确率 > 整张图直接发送 | 用 Playwright 截图 + 设计稿，对比整图 vs 局部裁剪的识别结果 | 局部裁剪识别准确率 ≥ 整图的 1.3 倍（特别在文字/UI 细节上） | ZeroZ-lab | 1 周 |
| H3 | 开发者愿意为远端 AI 功能（OCR/analyze/semantic-diff）付费 | Phase 2 后 survey 付费意愿 | ≥30% 活跃用户表示愿意付费 | ZeroZ-lab | 4 周 |
| H4 | 模型视觉能力增强不会让此工具失去价值 | 持续跟踪模型视觉输入文档 | 即使模型支持更高分辨率，成本/隐私/坐标映射/确定性仍需要本地前处理 | ZeroZ-lab | 持续 |

---

## 下一步行动

> ⚠️ 已在第4轮更新，详见底部「最终下一步行动（第4轮更新）」

| 行动 | 负责人 | 截止时间 | 完成标准 |
|------|--------|---------|---------|
| 实现 Phase 1：inspect/overview/tile/viewport/resize/rotate | ZeroZ-lab | 2 周 | 所有命令 JSON 输出 + 坐标映射 |
| 真实 AI Coding 场景验证 | ZeroZ-lab | 4 周 | ≥3 个前端任务完成截图→裁剪→定位→修复闭环 |
| Phase 2：concat/blur/diff + login 系统 | ZeroZ-lab | 5 周 | 登录流程跑通 + 跨平台构建 |
| Phase 3：远端 AI（analyze/ocr/semantic-diff） | ZeroZ-lab | 7 周 | 需 login 的 AI 功能可用 |
| 收集 3 个早期用户反馈 | ZeroZ-lab | 8 周 | 确认 H1/H3 |

**决策标准**：
- **继续**：H1 + H2 验证通过 → 进入 Phase 2
- **调整**：H1 失败但 H2 通过 → 考虑做 CI 工具
- **停止**：H1 + H2 都不通过 → 放弃

---

## 产品定位备忘

> **这不是图片工具，而是 Agent 的眼睛控制器。**
>
> 一句话：A CLI that lets AI agents inspect, navigate, crop, tile, and understand images — locally for free, with cloud AI features after login.
>
> 中文：给 AI Agent 用的本地图片视野控制工具。基础功能免费离线，高级 AI 功能需登录。

## 商业化路径备忘

```
Phase 1：开源 CLI — L1 本地视野控制（免费）
Phase 2：登录系统 + 本地补全（免费）
Phase 3：远端 AI — L2 analyze/ocr/semantic-diff（付费）
Phase 4：生态集成 — Claude Code Skill + GitHub Action + report.html
Phase 5：团队订阅 — CI 集成 + team baseline + 高级报告
```

## 护城河备忘

普通裁剪没有护城河。真正有护城河的是：
1. **可递归视野探索**（overview → tile → zoom in → 再 tile）
2. **稳定坐标系统**（rect / percent / anchor 统一坐标 + 映射）
3. **Agent-safe 文件系统**（不覆盖源文件、路径 sandbox、限制 tile 数）
4. **本地/远端分层**（免费本地 L1 + 登录解锁远端 AI L2 → 天然付费转化漏斗）
5. **Visual diff + JSON report**（第二阶段，把 diff 结果变成 Agent 可推理的结构化数据）

---

## 竞品分析（第2轮探索新增）

### 直接竞品

| 项目 | 语言 | Stars | 核心能力 | 与我们的差距 |
|------|------|-------|---------|------------|
| **[imgctl](https://github.com/agent-rt/imgctl)** | Rust | 0（2026.04） | 16命令：crop/slice/resize/diff/hash/map-coords/concat/blur/text/arrow/fix | 无 overview、无 viewport 导航、无递归探索、坐标不随 crop 输出、无 MCP |

### 生态相关工具（30+，按相关度排序）

| 类别 | 项目 | Stars | 做什么 | 与我们的关系 |
|------|------|-------|--------|------------|
| 屏幕捕获 | [Peekaboo](https://github.com/openclaw/Peekaboo) | ~4,600 | macOS 截图+视觉Q&A+GUI自动化 | 上游：它截图，我们处理截图 |
| 浏览器Agent | [gsd-browser](https://github.com/gsd-build/gsd-browser) | ~235 | Rust浏览器CLI，63命令，含zoom-region | 竞争+互补：浏览器场景重叠，但只处理网页不处理任意图片 |
| 图片处理MCP | [ImageSorcery](https://github.com/sunriseapps/imagesorcery-mcp) | ~314 | OpenCV图像处理 | 下游替代：可用它做底层，但缺 viewport 抽象 |
| 图片处理MCP | [PixelPanda](https://github.com/RyanKramer/pixelpanda-mcp) | ~21 | 33个图片工具 | 竞品：resize/crop/blur等，但无坐标映射和视野导航 |
| 视觉理解 | [Luma MCP](https://github.com/JochenYang/luma-mcp) | ~63 | 多模型视觉理解，大图自动分块 | 互补：它理解图片，我们控制 Agent 看图片的方式 |
| 视觉记忆 | [AgenticVision](https://github.com/agentralabs/agentic-vision) | ~6 | 截图+CLIP嵌入+diff+recall | 不同赛道：做视觉记忆，不做视野控制 |
| 浏览器视觉 | [BrowserLens](https://github.com/MAMISHO/browserlens) | 0 | Playwright+Ollama本地视觉 | 上游：截图来源 |
| 视觉回归 | [screentest-mcp](https://github.com/marilynceo/screentest-mcp) | 0 | MCP视觉回归测试 | 不同赛道：做断言，不做探索 |

### 真正的 gap（没有任何现有工具覆盖）

**没有工具提供"可导航的视觉视野"这个统一抽象层。** 现有工具：
- 要么做截图捕获（Peekaboo/gsd-browser）
- 要么做图片处理（imgctl/PixelPanda/ImageSorcery）
- 要么做视觉理解（Luma MCP/AgenticVision）

没有人做：`大图 → 总览 → 分块 → 局部放大 → 再放大 → 坐标映射 → Agent 推理` 这个递归导航。

---

## 场景验证（第2轮探索新增）

### 证据强度

| 场景 | 证据强度 | 关键证据 |
|------|---------|---------|
| **AI Coding 截图检查** | **STRONG** | Claude Code 12个月内 20+ issues。用户被迫手动截图→粘贴到聊天→再让 Agent 看。Cursor 已原生支持 UI 预览，Claude Code 没有 |
| **长截图阅读** | **STRONG** | Anthropic 官方：>1568px 自动缩放，>8000px 拒绝。用户报告长图导致终端崩溃 |
| **图片 token 成本** | **STRONG** | 官方公式 `tokens = w×h/750`。用户因图片 token 爆量被永久限流 |
| **Design-to-Code** | **MEDIUM** | 1568px API限制导致设计稿细节丢失，但独立投诉帖少 |
| **Computer-use 预处理** | **MEDIUM** | 官方参考实现含截图管线，用户请求更好的方案 |

### 关键 GitHub Issues（AI Coding 截图检查）

| Issue | 标题 | 核心诉求 |
|-------|------|---------|
| [#35866](https://github.com/anthropics/claude-code/issues/35866) | read tool should reliably deliver image files to the model | Mega-issue，合并 20+ 相关 issues，12个月持续活跃 |
| [#58233](https://github.com/anthropics/claude-code/issues/58233) | Read tool downsamples PNG based on file size | 4K截图"糊得看不清"，pngquant 压缩后反而更清晰 |
| [#56236](https://github.com/anthropics/claude-code/issues/56236) | Image quality degradation | "Claude 基于降质预览做决策，不是基于实际资产" |
| [#60559](https://github.com/anthropics/claude-code/issues/60559) | In-app UI preview with visual review | "Cursor 支持原生 UI 预览，Claude Code 不行，显著摩擦" |
| [#57034](https://github.com/anthropics/claude-code/issues/57034) | Support VS Code browser-sharing API | "截图→粘贴→让 Agent 看的工作流太笨重" |
| [#60810](https://github.com/anthropics/claude-code/issues/60810) | Support image viewing capability | 列举 UI截图/架构图/视觉调试/测试截图等用例 |

### 策略调整建议

**imgctl 的存在改变了一个假设**：做"又一个 Agent 图片处理 CLI"没有差异化空间。

**新的差异化聚焦**应该从"图片处理"转向"视野导航"：
1. 不是做 crop/slice/resize（imgctl 已经做了）
2. 而是做 **inspect → overview → navigate → zoom → coordinate-map** 这个导航流
3. 每一步操作都带坐标上下文，让 Agent 知道"我在看图片的哪个部分"
4. 支持递归探索：在大图上 tile → 选一块 → 再 tile → 再选

**这可能是比 imgctl 更高层次的抽象，就像浏览器 devtools 的 Elements 面板 vs DOM API 的区别。**

---

## 竞品深挖：imgctl（第3轮探索新增）

### imgctl 完整画像

| 维度 | 详情 |
|------|------|
| 版本 | v0.1.0，"all 16 commands shipped end-to-end" |
| 语言 | Rust，4 library crates + CLI binary |
| 平台 | macOS Apple Silicon only |
| Stars/Forks | 0 / 0 / 0 |
| 发布 | 无 GitHub Releases（源码构建） |
| 社区 | 0 adoption，1人项目 |
| 仓库活跃 | 22个repo，7周内创建（Apr 15 - Jun 1, 2026），产出速度极快 |
| 许可 | MIT OR Apache-2.0 |

### 16个命令完整清单

| # | 命令 | 功能 | 核心参数 |
|---|------|------|---------|
| 1 | `convert` | 格式转换 | `-i in -o out [--quality N]` |
| 2 | `resize` | 缩放 | `-i in -o out --width N [--fit contain\|...]` |
| 3 | `crop` | 裁剪 | `-i in -o out --x/y/w/h`（负数=从右/底计算） |
| 4 | `rect` | 矩形标注 | `--x/y/w/h --color HEX` |
| 5 | `text` | 文字叠加 | `--text STR --font STR --x/y --size N` |
| 6 | `arrow` | 箭头标注 | `--from x,y --to x,y` |
| 7 | `blur` | 模糊/打码 | `--region x,y,w,h --sigma N [--type pixelate]` |
| 8 | `concat` | 拼接 | `--direction horiz\|vert [--gap N]` |
| 9 | `annotate` | 批量操作 | `--config ops.json`（JSON-Schema 自省） |
| 10 | `info` | 元数据 | dimensions + EXIF + dominant colors |
| 11 | `diff` | 像素差异 | 像素 diff + bounding boxes |
| 12 | `hash` | 感知哈希 | phash + similarity score |
| 13 | `slice` | 分块切割 | `--rows N --cols N --output-dir dir/` |
| 14 | `map-coords` | 坐标映射 | `--from-size WxH --to-size WxH --point x,y` |
| 15 | `fix` | 格式修复 | JPEG truncation repair |
| 16 | `mermaid` | Mermaid渲染 | 需 Chrome/Chromium |

### imgctl vs. Agent Image Viewport 能力差距

| 能力 | imgctl | 我们需要 | 差距 |
|------|--------|---------|------|
| 格式转换/缩放/裁剪 | ✅ 完整 | 基础需要 | 无差距 |
| 矩形/箭头/文字标注 | ✅ 完整 | 锦上添花 | 无差距 |
| 像素 diff | ✅ 有 | 第二阶段需要 | 无差距 |
| 感知哈希 | ✅ 独有 | 可选 | imgctl 优势 |
| **overview 总览** | ❌ 无 | **核心** | **完全缺失** |
| **viewport 导航（pan/zoom）** | ❌ 无 | **核心** | **完全缺失** |
| **递归视野探索** | ❌ 无 | **护城河** | **完全缺失** |
| **坐标映射随操作输出** | 部分 | **核心** | crop 不返回源图坐标 |
| **语义视觉理解** | ❌ 无 | L2 远端 AI | **完全缺失** |
| **登录 + 云 AI 服务** | ❌ 无 | **商业化核心** | **完全缺失** |
| **跨平台** | ❌ 仅 macOS | 必须 | Linux/Windows 未支持 |

### 竞争威胁评估

- **基础 viewport 命令**：imgctl 作者可在 1-2 周内添加（他已有 Point/Size/Region newtype + crop + map-coords）
- **远端 AI 层 + 登录系统 + 递归探索**：需要数月，且架构方向不同
- **真正护城河**：viewport 导航状态追踪 + 递归探索树 + 本地/远端分层 + 登录制付费转化

---

## 技术验证（第3轮探索新增，第4轮更新）

### 推荐 Rust 技术栈

| 组件 | Crate | 用途 | 备注 |
|------|-------|------|------|
| 图片处理 | `image` 0.25.x | 主依赖，纯 Rust | 支持 PNG/JPEG/WebP/TIFF/AVIF/GIF 等 |
| SVG 渲染 | `resvg` + `tiny_skia` | 可选，SVG 转栅格 | 纯 Rust |
| CLI 框架 | `clap` | 参数解析 | |
| EXIF 解析 | `kamadak-exif` | EXIF 字段提取 | |
| HTTP 客户端（远端 AI） | `reqwest` | Phase 3 调用远端 API | |
| Auth（登录） | OAuth2 + 本地 token | Phase 2 login 流程 | |
| 分发 | `dist` (cargo-dist) | 多平台 CI + 安装器 | |

### 不推荐

- `turbojpeg`：C 依赖破坏单二进制分发
- libvips Rust 绑定：不存在成熟绑定

### 性能预估（6000×4000 图片，image-rs）

| 操作 | 预估时间 |
|------|---------|
| 读 JPEG | 50-150ms |
| 缩放到 800×600 (Lanczos3) | 100-200ms |
| 裁剪 1000×1000 区域 | 2-5ms |
| 高斯模糊 (sigma=3.0) | 100-300ms |
| 旋转 90° | 5-10ms |
| 仅读 dimensions | <1ms |
| 提取 EXIF | <5ms |

> 对 MCP 场景（Agent 按需调用，非批量），image-rs 性能绰绰有余。

### 二进制大小预估

| 配置 | 大小 |
|------|------|
| 本地 CLI（image + clap，Phase 1） | 3-5 MB |
| 含远端 AI（+ reqwest + auth，Phase 3） | 6-10 MB |
| 对比：ImageMagick 30-50 MB，Sharp 20-30 MB | |

> ⚠️ MCP server 已从架构中移除（第4轮决策）。CLI-only 架构不需要 rmcp/tokio，二进制更小。

---

## 用户痛点深挖（第3轮探索新增）

### 🔴 发现：问题比预期严重得多

**Claude Code 80+ 相关 GitHub issues**，远超第2轮发现的 20+。分为四大主题：

#### 主题 A：Agent 是"瞎子"——写 UI 代码但无法验证

| Issue | 评论 | 核心引述 |
|-------|------|---------|
| [#49603](https://github.com/anthropics/claude-code/issues/49603) | 3 | "After every commit I reported 'shipped, verified'. The user could not see any features. I never confirmed visual presence -- only ran headless DOM queries." |
| [#62604](https://github.com/anthropics/claude-code/issues/62604) | 2 | "Sub-agents declare work as PASS without performing actual verification" |
| [#63573](https://github.com/anthropics/claude-code/issues/63573) | 1 | "Claude Code continuously avoiding visual check" |
| [#58376](https://github.com/anthropics/claude-code/issues/58376) | 1 | "Claude lied about visual differences after looking at user-provided screenshots" |

#### 主题 B：图片处理会"杀死"整个会话（10+ issues）

| Issue | 评论 | 核心问题 |
|-------|------|---------|
| [#34566](https://github.com/anthropics/claude-code/issues/34566) | 12 | 图片 resize 失败后静默传入超大原图，永久破坏后续所有 API 调用 |
| [#2939](https://github.com/anthropics/claude-code/issues/2939) | **39** | 最高评论数图片 issue。5MB 限制导致永久会话失败 |
| [#43056](https://github.com/anthropics/claude-code/issues/43056) | 14 | 图片 base64 在历史中累积，最终超过 20MB 限制 |
| [#2104](https://github.com/anthropics/claude-code/issues/2104) | 20 | 硬 5MB 限制阻止正常截图 |
| [#42256](https://github.com/anthropics/claude-code/issues/42256) | 9 | Read tool 在每条后续消息中重发同一张超大图 |

#### 主题 C：CLI/终端无法输入图片

| Issue | 评论 | 核心问题 |
|-------|------|---------|
| [#12644](https://github.com/anthropics/claude-code/issues/12644) | **22** | CLI 无法接受截图，被迫切换到网页界面 |
| [#5277](https://github.com/anthropics/claude-code/issues/5277) | 16 | SSH/远程用户完全无法分享图片 |
| [#32005](https://github.com/anthropics/claude-code/issues/32005) | 11 | 终端用户无法粘贴截图 |

#### 主题 D：社区已经开始自建解决方案

| 工具 | Stars | 描述 |
|------|-------|------|
| **[ProofShot](https://github.com/AmElmo/proofshot)** | **826** | "Give AI coding agents eyes" — 录制浏览器会话、捕获截图、收集错误，打包验证产物。HN 161分、106评论 |
| **vibe** | — | Claude Code 自定义命令：截屏区域 + git status/diffs + 终端日志 → 给 Claude 调试 UI |

#### 关键 HN 引述

> **simonw**（知名开发者）在 Claude Code 2.0 讨论（842分）中：
> "Claude Code, Codex CLI etc can effectively do anything that a human could do by typing commands into a computer. **They still don't have good visual verification though.**"

> **blurjp**（vibe 工具作者）：
> "I built this because I was tired of describing UI bugs to Claude Code when I could just show them. You end up typing 'the button is misaligned by about 3 pixels' when you could just show a screenshot."

### 证据强度升级

| 场景 | 第2轮评估 | 第3轮评估 | 变化原因 |
|------|---------|---------|---------|
| AI Coding 截图检查 | STRONG | **VERY STRONG** | 80+ issues（非20+）、ProofShot 826 stars、HN 高分讨论 |
| 图片 token / 会话破坏 | — | **VERY STRONG** | 10+ session-bricking bugs，图片导致永久会话失败 |
| Agent "瞎子"问题 | — | **STRONG** | Agent 声称已完成但从未视觉验证（#49603） |

---

## 产品形态探索（第3轮探索，第4轮更新为 CLI-only）

> ⚠️ 第3轮原评估 MCP Server 为最优形态。第4轮决策改为 CLI-only + 登录制解锁远端 AI。
> 详细架构见底部「架构决策：CLI-only + 登录制解锁远端 AI（第4轮决策）」。

### 产品形态评估结论

| 形态 | 决策 | 理由 |
|------|------|------|
| **纯 CLI** | ✅ 采用 | Agent 用 Bash 调用，零配置，兼容性最广 |
| MCP Server | ❌ 放弃 | 增加维护负担，CLI JSON 输出已是 Agent-native |
| Claude Code Skill | 📋 Phase 4 | 作为分发加速器（SKILL.md 教 Agent 怎么用 CLI），不是产品本体 |
| GitHub Action | 📋 Phase 4 | CI 视觉回归，商业化扩展 |

---

## 第3轮探索结论

### 关键发现

1. **痛点比预期严重 4 倍**：80+ GitHub issues（非20+），session-bricking bugs，Agent "声称完成但从未看一眼 UI"
2. **ProofShot（826 stars）验证市场需求**：专门解决"AI Agent 的眼睛"问题，HN 161分
3. **imgctl 竞争可控**：0 stars、macOS only、1人项目。但它可能在 1-2 周内添加基础 viewport 命令
4. **技术栈完全可行**：image-rs + clap + dist，3-5MB 单二进制（Phase 1）

### 策略调整

- **差异化聚焦**：viewport 导航 + 坐标系统 + 递归探索（imgctl 不做的）
- **首发场景不变**：AI Coding Screenshot Inspector

---

## 架构决策：CLI-only + 登录制解锁远端 AI（第4轮决策）

### 决策：放弃 MCP，纯 CLI 架构

**理由**：
1. Agent 天然会用 Bash 跑命令（Claude Code、Cursor、Codex 都行），JSON 输出的 CLI 本身就是 Agent-native
2. MCP 增加维护负担（CLI + MCP schema 两套），收益有限
3. 用户不需要学怎么配 MCP server，`brew install` 就能用
4. CLI 兼容范围最广：任何 Agent、任何脚本、任何 CI、任何终端

### 分层架构

```
image-viewport（单二进制）
│
├── 免登录（本地，离线，确定性）
│   ├── inspect      < 1ms     读元数据
│   ├── overview     ~0.2s     总览缩放
│   ├── tile         ~5ms/块   网格切割
│   ├── viewport     ~5ms     局部裁剪（anchor/percent/rect）
│   ├── resize       ~0.2s     缩放
│   ├── rotate       ~10ms     旋转
│   ├── concat       ~10ms     拼接
│   ├── blur         ~80ms     模糊/打码
│   └── diff         ~150ms    像素级差异
│
├── 需登录（远端 AI，需网络）
│   ├── analyze      UI 组件/布局识别
│   ├── ocr          文字提取
│   ├── semantic-diff  语义级差异
│   └── smart-crop   主体感知裁剪
│
└── 登录管理
    ├── login            OAuth 浏览器登录
    ├── login --api-key  CI/headless 用
    ├── whoami           查看当前账户
    └── logout
```

### 登录流程

```bash
# 交互式（开发者日常）
$ image-viewport login
Opening browser... please authorize.
✓ Logged in as user@example.com

# CI / headless
$ image-viewport login --api-key iv_sk_xxxxx
✓ API key saved

# 环境变量（最轻量）
$ IMAGE_VIEWPORT_API_KEY=iv_sk_xxxxx image-viewport analyze screenshot.png
```

### 商业化分层

| 层 | 内容 | 定价 | 逻辑 |
|---|------|------|------|
| L1 本地 | inspect/overview/tile/viewport/resize/rotate/concat/blur/diff | 永久免费开源 | 确定性操作无边际成本 |
| L2 远端 | analyze/ocr/semantic-diff/smart-crop | 按用量或订阅 | 有 API 成本 |
| L3 闭环 | Agent 验收流程 + CI 集成 + report.html | 团队订阅 | 组合价值 |

### CLI-only 的好处

**Agent 零配置**：不需要教 Agent 配 MCP。它会跑 Bash 就行。
```bash
image-viewport inspect screenshot.png --json
image-viewport tile screenshot.png --rows 4 --cols 3 --out-dir ./tiles --json
image-viewport viewport anchor screenshot.png focus.png --anchor right --width 1500 --json
```

**开发者日常也能用**：不在 AI Coding 场景里，手动画图也行。
```bash
image-viewport overview design.png overview.jpg --max-width 1200
image-viewport diff before.png after.png --json
```

**CI/CD 也能用**：
```yaml
- run: |
    image-viewport diff baseline.png current.png --json > diff-result.json
    image-viewport login --api-key ${{ secrets.IV_API_KEY }}
    image-viewport semantic-diff baseline.png current.png --json > semantic.json
```

**渐进式付费转化**：用户装上 → 免费用 inspect/tile/viewport → 真的有用 → 某天需要 OCR → `image-viewport login` → 付费。

### 与 imgctl 差异化

| | imgctl | 我们 |
|--|--------|------|
| 本地层 | 16个命令（覆盖很好） | 8个 viewport 专注命令 |
| 远端 AI 层 | ❌ 无 | ✅ analyze / ocr / semantic-diff / smart-crop |
| 分层架构 | ❌ 扁平 | ✅ L1本地 / L2远端 / L3闭环 |
| 登录 + 云服务 | ❌ 无 | ✅ login + 托管 AI 服务 |
| 商业化 | ❌ 无路径 | ✅ 天然免费/付费分层 |

---

## 最终 MVP 定义（第4轮更新）

### Phase 1（2周）：本地 CLI 核心 — L1 视野控制

**核心功能**（去掉任何一个就不成立）：
1. `inspect` — 图片 metadata + 建议操作
2. `overview` — 缩放总览图
3. `tile` — 网格分块（含坐标映射）
4. `viewport` — 局部裁剪（anchor / percent / rect，含坐标映射）
5. JSON-first 输出 + 稳定坐标系统

**边界声明**：
- 不做远端 AI 功能（Phase 3）
- 不做 login（Phase 2）
- 不做 diff / compare（Phase 2）
- 不做 MCP server（不做，用 CLI-only）
- 不做 IDE 插件 / UI

**验证标准**：
- Agent 能完成：截图 → inspect → overview → tile → viewport → 定位问题的完整闭环
- 所有命令输出稳定 JSON schema，包含坐标映射
- 3 个早期用户反馈"有用"

### Phase 2（+1周）：本地 CLI 补全 + 登录系统

- `concat` / `blur` / `diff`（像素级）
- `login` / `whoami` / `logout`
- 跨平台构建（macOS ARM+Intel / Linux / Windows）

### Phase 3（+2周）：远端 AI 功能 — L2

- `analyze`（UI 组件/布局识别）
- `ocr`（文字提取）
- `semantic-diff`（语义级差异）
- 需要 login 后才能用
- 用户自带 API key 或用托管服务

### Phase 4（+2周）：生态集成 — L3

- Claude Code Skill（SKILL.md，教 Agent 怎么用 CLI）
- GitHub Action（CI 视觉回归）
- report.html（可视化差异报告）
- → 商业化启动

---

## 最终假设清单（第4轮更新）

| # | 假设 | 验证方法 | 成功标准 | 时间 |
|---|------|---------|---------|------|
| H1 | AI Coding Agent 会通过 Bash 主动调用 CLI 控制视觉输入 | 构建 CLI，让 Claude Code 在真实前端任务中使用 | Agent 在 ≥50% 的前端修改任务中主动调用 inspect/tile/viewport | 2 周 |
| H2 | 局部裁剪后模型视觉识别准确率 > 整张图直接发送 | 对比整图 vs 局部裁剪的识别结果 | 局部裁剪识别准确率 ≥ 整图的 1.3 倍 | 1 周 |
| H3 | 开发者愿意为远端 AI 功能（OCR/analyze/semantic-diff）付费 | Phase 2 后 survey 付费意愿 | ≥30% 活跃用户表示愿意付费 | 4 周 |
| H4 | 模型视觉能力增强不会让此工具失去价值 | 持续跟踪模型视觉输入文档 | 即使模型支持更高分辨率，成本/隐私/坐标映射/确定性仍需要本地前处理 | 持续 |

---

## 最终下一步行动（第4轮更新）

| 行动 | 截止时间 | 完成标准 |
|------|---------|---------|
| 实现 Phase 1：inspect/overview/tile/viewport/resize/rotate | 2 周 | 所有命令 JSON 输出 + 坐标映射 |
| 真实 AI Coding 场景验证 | 4 周 | ≥3 个前端任务完成截图→裁剪→定位→修复闭环 |
| Phase 2：concat/blur/diff + login 系统 | 5 周 | 登录流程跑通 + 跨平台构建 |
| Phase 3：远端 AI（analyze/ocr/semantic-diff） | 7 周 | 需 login 的 AI 功能可用 |
| 收集 3 个早期用户反馈 | 8 周 | 确认 H1/H3 |

**决策标准**：
- **继续**：H1 + H2 验证通过 → 进入 Phase 2
- **调整**：H1 失败但 H2 通过 → 考虑做 CI 工具
- **停止**：H1 + H2 都不通过 → 放弃

---

## 最终技术栈（第4轮更新）

| 组件 | Crate | 用途 |
|------|-------|------|
| 图片处理 | `image` 0.25.x | 主依赖，纯 Rust |
| CLI 框架 | `clap` | 参数解析 |
| EXIF 解析 | `kamadak-exif` | EXIF 字段提取 |
| HTTP 客户端（远端 AI） | `reqwest` | 调用远端 API |
| Auth | OAuth2 + 本地 token 存储 | login 流程 |
| 分发 | `dist` (cargo-dist) | 多平台 CI + 安装器 |
| 二进制大小 | 目标 5-8 MB（不含远端） | 对比 ImageMagick 30-50 MB |

**不引入的依赖**：rmcp（不做 MCP）、turbojpeg（C 依赖）、libvips（无 Rust 绑定）、tokio（CLI-only 用不着 async runtime）
