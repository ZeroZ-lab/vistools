# vistools

> **给 AI 编程助手的坐标可信视觉调试协议。**

检视、导航、裁剪、取色大图片 — 每条命令返回结构化 JSON，生成视野时附带指回源图的坐标映射。

[English](README.md) | **中文文档**

```
$ vistools inspect screenshot.png
{
  "ok": true,
  "data": {
    "source": { "width": 3200, "height": 2400, "format": "png", "size_bytes": 808243 },
    "suggestion": {
      "needs_overview": true,
      "max_tile_rows": 2,
      "max_tile_cols": 3,
      "recommended_next": "overview",
      "reason": "long side 3200 exceeds 1568 visual threshold",
      "suggested_max_side": 1568
    }
  }
}
```

## Before vs After

**没有 vistools：**
```
Agent 读取 3200×2400 截图 → 压缩后细节丢失
  → 声称"按钮看起来正确" → 无法验证
```

**使用 vistools：**
```
1. inspect → 3200×2400，需要 overview
2. overview --max-side 1200 → 缩小预览，scale_factor = 0.375
3. 在 overview 中发现异常 (800, 600) → 映射回源图：(2133, 1600)
4. viewport rect → 精确裁剪该区域
5. sample --x 2133 --y 1600 → 颜色是 #e74c3c，不是预期的 #2563eb
6. 报告："源图坐标 (2133, 1600) 处的按钮背景颜色错误"
```

## 为什么需要 vistools

当 AI Agent（Claude Code、Cursor、Codex、浏览器 Agent）收到一张大截图或设计稿时，通常只能一次性看到整张图 — 被压缩、被缩小、分辨率不够用。`vistools` 给 Agent 提供和人类一样的操作方式：先看全貌，选定感兴趣的区域，放大，读取细节。

三个设计原则贯穿始终：

- **JSON-first** — 每条命令输出统一的 `CommandResult<T>` 信封，成功或失败都是同一个结构，Agent 永远用同样的方式解析。
- **坐标映射** — 每个生成视野都附带 `coordinate_mapping`，描述输出坐标如何映射回源图。Agent 在裁剪图中找到按钮，就能精确定位到原图中的位置。
- **Agent-safe** — 绝不覆盖源文件。路径沙箱（禁止 `..` 逃逸）。像素上限（100MP）和 tile 上限（64）防止失控调用。

## 安装

### Claude Code 插件（推荐）

```bash
# 在 Claude Code 中：
/plugin install https://github.com/ZeroZ-lab/vistools-skills
# 然后：/vistools screenshot.png
```

### 从源码安装（Rust 1.88+）

```bash
git clone https://github.com/ZeroZ-lab/vistools
cd vistools
cargo install --path crates/cli   # 安装到 ~/.cargo/bin/vistools
```

也可以直接编译运行：

```bash
cargo build --release
./target/release/vistools <命令>
```

发布二进制是单个 ~5MB 的独立可执行文件，无运行时依赖。

## 命令一览

| 命令 | 用途 | 示例 |
|------|------|------|
| `inspect` | 读取元数据 + 策略建议 | `vistools inspect img.png` |
| `overview` | 缩小预览 | `vistools overview img.png out.png --max-side 1200` |
| `tile` | 网格切图 | `vistools tile img.png --rows 2 --cols 3 --out-dir ./tiles` |
| `viewport` | 裁剪区域（3 种模式） | `vistools viewport anchor img.png out.png --anchor center --width 800 --height 600` |
| `sample` | 点/区域取色 | `vistools sample img.png --x 120 --y 80` |

## 命令详解

### `inspect` — 元数据 + 策略建议

拿到未知图片后的第一步。只读文件头，亚毫秒级完成。

```bash
vistools inspect large_screenshot.png
```

当长边超过 1568px（Claude 视觉模型阈值），`suggestion.recommended_next` 为 `overview`；否则为 `direct`。`max_tile_rows` / `max_tile_cols` 告诉你需要全图覆盖时该用多细的网格。

### `overview` — 缩小预览

```bash
vistools overview large_screenshot.png overview.png --max-side 1200
```

缩放到长边不超过 `max_side`，保持宽高比。返回 `scale_factor`，可以把 overview 中的点击位置映射回源图。

### `tile` — 网格切图

```bash
vistools tile large_screenshot.png --rows 2 --cols 3 --out-dir ./tiles
```

生成 `row-N-col-M.<ext>` 文件。每行/列最后一个 tile 吸收余数像素，保证 tile 精确覆盖整张源图。

### `viewport` — 裁剪区域

三种模式，输出结构相同：

```bash
# 锚点模式（九宫格：top-left、center、bottom-right 等）
vistools viewport anchor src.png crop.png --anchor top-right --width 800 --height 600

# 百分比模式（源图的分数坐标）
vistools viewport percent src.png crop.png --x 0.3 --y 0.3 --w 0.4 --h 0.4

# 像素矩形模式
vistools viewport rect src.png crop.png --x 1100 --y 200 --width 700 --height 700
```

百分比模式严格校验：`x/y/w/h` 必须在 `0..1` 内，且 `x + w` / `y + h` 不能超过 `1`。

### `sample` — 点/区域取色器

```bash
# 单点颜色
vistools sample src.png --x 120 --y 80

# 区域平均色和 alpha 统计
vistools sample src.png --rect 100,80,40,40
```

点模式返回 `rgba`、`rgb`、小写 `hex` 和 `alpha`。区域模式返回四通道平均色、`alpha_stats`（`min`、`max`、`average`、`transparent_ratio`）和 `pixel_count`。`sample` 只读源图，不生成输出图片。

### 帮助与版本

```bash
vistools --help              # 列出所有命令和简要说明
vistools inspect --help      # 查看子命令详细帮助
vistools --version           # 输出版本号（如 "vistools 0.2.0"）
```

## JSON 输出

每条命令 — 无论成功还是失败 — 都在 stdout 输出相同结构的 JSON：

```json
{
  "ok": true,
  "operation": "viewport",
  "input": "src.png",
  "data": {
    "output": "crop.png",
    "source": { "width": 3200, "height": 2400, "format": "png", "size_bytes": 808243 },
    "crop": {
      "mode": "anchor",
      "region": { "x": 2200, "y": 0, "width": 1000, "height": 600 },
      "params": { "anchor": "TopRight", "width": 1000, "height": 600 }
    },
    "result": { "width": 1000, "height": 600 },
    "coordinate_mapping": {
      "crop_origin_in_source": [2200, 0],
      "scale_factor": null,
      "formula": "source_x = result_x + 2200, source_y = result_y"
    }
  },
  "warnings": [],
  "elapsed_ms": 12
}
```

失败时 `ok` 为 `false`，`data` 不存在，`error` 携带稳定的机器可读 `code`：

```json
{
  "ok": false,
  "operation": "inspect",
  "input": "/tmp/nope.png",
  "error": { "code": "FILE_NOT_FOUND", "message": "input file not found: /tmp/nope.png" },
  "warnings": [],
  "elapsed_ms": 0
}
```

失败时进程退出码也为非零。

### 错误码

| 错误码 | 含义 |
|--------|------|
| `FILE_NOT_FOUND` | 输入文件不存在或不是普通文件 |
| `UNSUPPORTED_FORMAT` | 图片解码器无法读取 |
| `INVALID_DIMENSIONS` | 宽或高为 0 |
| `INVALID_COORDINATES` | viewport/sample 点或矩形超出源图边界 |
| `INVALID_PARAMETERS` | tile 数量 > 64、max side 为 0、sample 模式参数错误等 |
| `OUTPUT_WRITE_ERROR` | 无法写入输出文件 |
| `PATH_ESCAPE` | 路径包含 `..` |
| `OUTPUT_SAME_AS_INPUT` | 输出路径会覆盖源文件 |
| `PIXEL_LIMIT_EXCEEDED` | 源图超过 1 亿像素 |

## Agent 典型工作流

```
1. inspect src.png            # 大图？建议什么网格？
       │
       ▼  needs_overview=true
2. overview src.png overview.png --max-side 1200
       │
       ▼  在 overview 中找到感兴趣区域
3a. tile src.png --rows 2 --cols 3 --out-dir ./tiles
       │
       ▼  或者，已知大概位置：
3b. viewport anchor src.png crop.png --anchor top-right --width 800 --height 600
       │
       ▼  coordinate_mapping 告诉你裁剪图中 (100, 50) 在源图的哪里
4. sample src.png --x 1110 --y 800
       │
       ▼  读取源图坐标上的精确颜色和透明度
5. Agent 处理裁剪图
```

`coordinate_mapping.formula` 是机器可读的映射公式：

```
source_x = result_x + 2200, source_y = result_y          # 裁剪
source_x = result_x / 0.375000, source_y = result_y / 0.375000   # overview
```

## Skills

Skills 维护在 [ZeroZ-lab/vistools-skills](https://github.com/ZeroZ-lab/vistools-skills) 仓库。

```bash
# Claude Code — 从 skills 仓库安装（轻量，不含 Rust 源码）
/plugin install https://github.com/ZeroZ-lab/vistools-skills

# 然后使用：/vistools screenshot.png
```

支持 Claude Code、Cursor、Codex。

## 开发

```bash
cargo test                        # 单元 + 集成测试
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo build --release             # 发布构建（~5MB，LTO + strip）
```

支持的输入格式：PNG、JPEG、WebP、TIFF、BMP、GIF。
输出格式根据输出文件扩展名自动推断。

## 项目结构

```
vistools/
├── crates/
│   ├── core/            # 核心库：types、guard、coord、每个命令一个模块
│   └── cli/             # 薄 clap 包装 + 集成测试
├── fixtures/            # 单元测试图片（64x64、256x256、1000x1000）
│   └── e2e/             # 真实世界测试图片
└── docs/                # 设计决策（project.md）、时间线、合约
```

## 许可证

MIT / Apache-2.0，任选其一。
