# Idea Brief — 摄影计量能力（Photography Metering）

> 探索阶段的产出：在现有 photo.rs 测量基础上，扩展为完整的摄影行业计量命令面。

## 版本信息

| 项目 | 值 |
|------|-----|
| 日期 | 2026-06-02 |
| 参与者 | ZeroZ-lab |
| 探索轮次 | 1（方向明确，1 轮收敛） |

---

## 痛点

**一句话描述**：Agent 帮摄影师批量质检照片时，缺少摄影语言级别的计量命令——不能回答"曝光对不对"、"对焦准不准"、"颜色对不对"。

**具体场景**：摄影师拍了一场活动，800 张照片需要入库。Agent 逐张检查：这张高光溢出 40%、那整体欠曝 1.5 档、还有一张焦点不在主体上。当前 vistools 的 photo.rs 能做基础的 clipping/sharpness/color-cast 检测，但缺少摄影行业的核心计量概念——RGB 通道直方图、Zone System 分区、EV 估算、焦点分布地图、色温估算。

**谁受影响**：
- 摄影师（批量质检、入库筛选）
- 照片编辑/图库管理员（质量门槛控制）
- Agent 开发者（编排自动化摄影工作流）

**影响程度**：中频 × 中高严重度。摄影师每次交付前都要做质检，但当前靠人眼或 Lightroom 手动筛选。

**当前方案 + 不足**：
- vistools photo.rs（6 个命令）：sharpness/histogram/clipping/contrast/color-cast——覆盖基础像素检测，缺少摄影语言
- Lightroom / Capture One：人眼判断，不可 Agent 化
- ImageMagick `identify -verbose`：原始数据，无摄影语义

**根因分析**（5 Whys）：
1. 为什么 Agent 做不好摄影质检？→ 缺少摄影语言级别的计量原语
2. 为什么现有 photo.rs 不够？→ 只有像素级指标，不能回答"曝光对不对"这种摄影问题
3. 为什么不直接用像素指标？→ "highlight clipping 3%"不如"Zone VIII 以上占 15%，高光细节丢失"有操作性
4. 为什么摄影语言重要？→ 摄影师的判断标准是 Zone/EV/色温，不是像素值
5. 为什么现在值得做？→ vistools 已有像素基础 + Agent 编排能力，差的就是语义层

---

## 方向地图

### 方向对比

| 方向 | 核心思路 | 适用场景 | 风险 | 验证难度 |
|------|---------|---------|------|---------|
| A: 影调计量（Tonal Metering） | Zone System + EV 估算 + 测光模式，回答"曝光对不对" | 所有摄影场景的曝光评估 | 假设 sRGB gamma，色彩空间感知有限 | 低 |
| B: 焦点地图（Focus Map） | 网格锐度分布，回答"对焦准不准" | 人像/产品摄影焦点检查、跑焦筛选 | 大图计算量大 | 中 |
| C: 色彩保真（Color Fidelity） | 色温估算 + 色域检查 + ΔE，回答"颜色对不对" | 产品摄影/电商色彩一致性、印刷色域合规 | 需要色彩空间转换（RGB→Lab） | 中高 |
| D: 噪点指纹（Noise Fingerprint） | SNR 估算 + 噪声类型识别，回答"画质够不够" | 高 ISO 照片质量评估、降噪前后对比 | 平坦区域检测本身是个难题 | 高 |

### 方向详情

**方向 A: 影调计量（Tonal Metering）**
- **做什么**：把现有 histogram/clipping 升级为摄影级影调分析——Zone System 10 区分布、EV 估算、4 种测光模式（evaluative/spot/center-weighted/highlight-weighted）、RGB 通道直方图
- **适用场景**：所有摄影场景都需要评估曝光——这是摄影师拿到照片后的第一个动作
- **优势**：纯像素数学，不依赖 ML；和现有 histogram/clipping 天然互补；摄影师一看就懂的概念（Zone V = 中灰、EV ±1.3 = 欠曝 1.3 档）
- **劣势**：Zone System 假设 sRGB gamma 曲线，对 AdobeRGB/P3 色彩空间的图会有偏差
- **参考产品**：Lightroom 直方图（RGB 三通道 + clipping 警告）、相机的测光系统
- **AI 研究发现**：现有 histogram 是亮度单通道，摄影师看的是 RGB 三通道——通道级 clipping 是摄影 QC 的关键

**方向 B: 焦点地图（Focus Map）**
- **做什么**：NxM 网格锐度矩阵，输出焦点位置 + 锐度热图 + 坐标映射
- **适用场景**：人像摄影确认眼睛是否对焦、产品摄影确认主体锐度、批量筛选跑焦照片
- **优势**：复用已有 sharpness 算法 + tile 基础设施；Agent 可组合 viewport 放大确认
- **劣势**：大图全网格计算量较大；需要定义"对焦准确"的阈值
- **参考产品**：FocusPeaker（实时峰值对焦）、RawDigger（锐度分析）
- **AI 研究发现**：现有 sharpness 是全局单点，不能回答"焦点在图片的哪个位置"

**方向 C: 色彩保真（Color Fidelity）**
- **做什么**：色温估算（Kelvin）+ 色调偏移（tint）+ 色域边界检查（sRGB/P3/AdobeRGB）+ ΔE 色差
- **适用场景**：产品摄影色彩一致性、电商图片色域合规、修图前后色差量化
- **优势**：比 color-cast 更专业（"偏暖 500K"比"红色通道偏高"更有操作性）；色域检查在电商/印刷有明确商业价值
- **劣势**：需要色彩空间转换（RGB → CIE Lab），需增加依赖或手写转换；ΔE 需要参考色
- **参考产品**：X-Rite ColorChecker（硬件色卡）、Lightroom 白平衡吸管
- **AI 研究发现**：现有 color-cast 只输出"dominant channel: red"，摄影师说的是"偏暖 500K"

**方向 D: 噪点指纹（Noise Fingerprint）**
- **做什么**：估算区域噪声标准差、SNR、噪声类型（高斯/带状）
- **适用场景**：高 ISO 照片质量评估、传感器缺陷检测、降噪前后对比
- **优势**：纯统计方法，不需要 ML；和 sharpness 互补（清晰 vs 噪声是摄影经典权衡）
- **劣势**：平坦区域检测本身是难题（怎么区分纹理和噪声？）；"相当于 ISO X"的标定需要训练数据
- **参考产品**：DxO DeepPRIME（AI 降噪）、Neat Image（噪声分析）
- **AI 研究发现**：噪声估算需要"无纹理参考区"，对自然照片不总是可行

---

## 推荐方向 + 评估

**选择**：方向 A（影调计量）为核心，B/C 作为次优先扩展

**理由**：
1. 曝光是摄影质检的第一判断，每个摄影师拿到照片先看直方图和曝光
2. 方向 A 实现复杂度最低（纯像素数学），且和现有命令互补度最高
3. 方向 B/C/D 可以按需追加，各自独立

**被拒方向及理由**：
- 紫边/暗角检测：需要边缘检测 + 统计模型，实现复杂度高，P2 以后再说
- 镜头畸变检测：需要标定参考或图案识别，超出纯像素计量范畴
- EXIF 深度解析：已有 kamadak-exif，可在 inspect 中增强，不属于计量命令

**评估矩阵**：

| 标准 | 权重 | 方向 A (影调计量) | 方向 B (焦点地图) | 方向 C (色彩保真) | 方向 D (噪点指纹) |
|------|------|-------------------|-------------------|-------------------|-------------------|
| 摄影师价值 | 高 | 9.5 | 8.5 | 8.0 | 7.0 |
| Agent 可操作性 | 高 | 9.0 | 8.5 | 8.0 | 7.5 |
| 实现复杂度 | 高 | 9.0 (低) | 7.0 (中) | 6.0 (中高) | 5.0 (高) |
| 和现有命令互补 | 中 | 9.5 | 8.0 | 8.5 | 6.5 |
| v1 闭环贡献 | 中 | 9.0 | 8.0 | 8.0 | 7.0 |
| **加权总分** | | **9.1** | **8.0** | **7.7** | **6.6** |

---

## MVP 定义

**核心功能**（去掉任何一个就不成立）：

### P0：影调计量核心（MVP 必须）

1. **`histogram` 增强**：+RGB 三通道直方图（R/G/B 各 256 bins）+ 每通道分位数（p5/p50/p95）+ 通道级 clipping 标记
2. **`zone-map` 新增**：亮度映射到 Zone System 0-X 区，输出每 zone 像素占比 + 代表区域坐标
3. **`exposure` 新增**：基于平均亮度估算 EV 偏移 + 4 种测光模式（evaluative / spot / center-weighted / highlight-weighted）

### P1：焦点 + 色彩扩展

4. **`focus-map` 新增**：NxM 网格锐度矩阵 + 最锐点坐标 + 焦点区域标注
5. **`white-balance` 新增**：从 RGB 均值估算灰世界 gains + warm/cool + green/magenta bias

### P2：进阶计量

6. `gamut` 新增：色域边界检查（sRGB / P3 / AdobeRGB）
7. `noise` 新增：区域噪声估算（SNR + 标准差）

**边界声明**（明确不做）：
- 不做紫边/暗角/畸变检测，因为实现复杂度高且需边缘检测模型
- 不做 EXIF 深度解析，这是 inspect 命令的增强，不属于计量层
- 不做降噪/锐化等像素修改操作，vistools 是测量工具不是处理工具
- 不做 AI 模型推理（主体检测、场景分类），保持纯像素数学
- 不做批量编排（"一次质检 800 张"），编排由 Agent 负责

**验证标准**（怎么算 MVP 成功）：
- Agent 能完成：inspect → histogram --rgb → exposure → zone-map → assert-exposure 完整闭环
- 摄影师看到 zone-map / exposure 输出能直接理解（Zone V 占 20%、欠曝 1.3 EV）
- 所有命令输出 CommandResult<T> 结构化 JSON + 坐标映射
- 和现有 photo.rs 6 个命令无冲突，向后兼容

---

## 假设清单

| # | 假设 | 验证方法 | 成功标准 | 时间 |
|---|------|---------|---------|------|
| H1 | RGB 直方图 + Zone System 能让 Agent 做出比亮度直方图更准确的曝光判断 | 用 50 张不同曝光的真实照片，对比 Agent 用 histogram vs histogram --rgb + zone-map 的判断准确率 | RGB + Zone 判断准确率 ≥ 亮度单通道的 1.2 倍 | 1 周 |
| H2 | EV 估算值和 Lightroom 的曝光评估一致 | 选 20 张照片，对比 vistools exposure 输出与 Lightroom 直方图判断 | EV 偏差 ≤ ±0.5 档 | 1 周 |
| H3 | 摄影师认可 Zone System 语义（而非像素指标）作为质检标准 | 3 个摄影师 review 输出 | ≥2 人说"这比我手动看直方图快" | 2 周 |
| H4 | 色温估算在 sRGB 色彩空间内足够准确 | 用 X-Rite ColorChecker 拍摄的参考图验证 | 色温估算误差 ≤ ±500K | 2 周 |

---

## 下一步行动

| 行动 | 负责人 | 截止时间 | 完成标准 |
|------|--------|---------|---------|
| histogram 增强（+RGB 通道） | ZeroZ-lab | 3 天 | R/G/B 三通道直方图 + 分位数输出 + 测试 |
| zone-map 实现 | ZeroZ-lab | 5 天 | 0-X 区分布 + 坐标 + 测试 |
| exposure 实现 | ZeroZ-lab | 5 天 | EV 估算 + 4 种测光模式 + 测试 |
| focus-map 实现 | ZeroZ-lab | 7 天 | NxM 网格锐度矩阵 + 坐标 + 测试 |
| white-balance 实现 | ZeroZ-lab | 7 天 | 灰世界 gains + bias 估算 + 测试 |
| 真实照片场景验证 | ZeroZ-lab | 10 天 | 50 张照片批量质检闭环 |

**决策标准**：
- **继续**：H1 + H2 验证通过 → 进入 define 阶段写 PRD/contract
- **调整**：H1 通过但 H2 不通过 → 简化 exposure 为定性判断（under/correct/over），不做 EV 数值
- **停止**：H1 不通过 → 摄影师不认可 Zone System 语义，重新评估方向

---

## 和现有命令的关系

| 现有命令 | 变化 | 说明 |
|---------|------|------|
| `histogram` | **增强** | +RGB 三通道，不破坏现有亮度直方图输出 |
| `highlight-clipping` | 保留 | 和 zone-map 互补（zone VIII+ vs clipping threshold） |
| `shadow-clipping` | 保留 | 和 zone-map 互补（zone II- vs clipping threshold） |
| `contrast` | 保留 | RMS 对比度仍有价值 |
| `sharpness` | 保留 | focus-map 复用其算法，但 focus-map 是网格化版本 |
| `color-cast` | 保留 | white-balance 是其摄影语言升级版，两者并存 |

---

## 命令清单总览

### 现有（photo.rs）

| 命令 | 层级 | 状态 | 度量 |
|------|------|------|------|
| `sharpness` | 测量层 | ✅ 已有 | 边缘梯度方差 |
| `histogram` | 测量层 | ✅ 已有 | 亮度直方图 |
| `highlight-clipping` | 测量层 | ✅ 已有 | 高光溢出检测 |
| `shadow-clipping` | 测量层 | ✅ 已有 | 暗部溢出检测 |
| `contrast` | 测量层 | ✅ 已有 | RMS 对比度 |
| `color-cast` | 测量层 | ✅ 已有 | 通道偏移检测 |

### 新增（摄影计量）

| 命令 | 层级 | 优先级 | 度量 |
|------|------|--------|------|
| `histogram`（增强） | 测量层 | P0 | +RGB 三通道直方图 + 分位数 |
| `zone-map` | 测量层 | P0 | Zone System 0-X 区分布 |
| `exposure` | 测量层 | P0 | EV 估算 + 测光模式 |
| `focus-map` | 测量层 | P1 | NxM 网格锐度矩阵 |
| `white-balance` | 测量层 | P1 | 灰世界 gains + warm/cool + green/magenta bias |
| `gamut` | 测量层 | P2 | 色域边界检查 |
| `noise` | 测量层 | P2 | SNR + 噪声估算 |

### Agent 批量质检工作流

```text
对每张照片:
  inspect                    → 元数据（尺寸/格式/EXIF）
  histogram --rgb            → RGB 三通道直方图
  exposure --mode evaluative → EV 估算 + 测光评估
  zone-map                   → 影调分区分布
  highlight-clipping         → 高光溢出像素
  shadow-clipping            → 暗部溢出像素
  sharpness                  → 全局锐度
  color-cast                 → 白平衡偏色
  ────────────────────────────────────────
  assert-exposure(ev > -1.0 && ev < 1.0)
  assert-clipping(ratio < 0.05)
  assert-sharpness(score > threshold)
  ────────────────────────────────────────
  → JSON 质检报告（pass/fail + 每项指标 + 坐标证据）
```
