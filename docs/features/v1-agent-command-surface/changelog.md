# Changelog

## 2026-06-02

- 新增 `contract.md`，定义 `vistools` v1 原子命令清单
- 将命令面收敛为三层：视野层 / 测量层 / 断言层
- 确认 v1 保留命令：`inspect` / `overview` / `tile` / `viewport` / `sample`
- 确认 v1 新增优先级：`diff` / `measure` / `assert-color` / `assert-diff` / `assert-region`
- 明确暂不进入 v1 的范围：行业命令、通用像素处理、智能识别、内建 workflow engine、自然语言报告

## 2026-06-02 — diff codegen

- 触发：用户调用 `forge:codegen`，要求直接生成下一个 v1 命令
- 版本：vistools `0.2.5` → `0.2.6`
- 说明：缺少专门 `diff` detail/plan，本次从 v1 命令面合约推导最小实现
- 新增命令：`vistools diff <EXPECTED> <ACTUAL> [--rect x,y,width,height]`
- 输出：`pixel_count`、`changed_pixels`、`changed_ratio`、`mean_delta`、`max_delta`、`bounding_rect`
- 边界：要求两张图尺寸一致；只输出 JSON，不生成 diff 图片
- 验证：157 tests passed / clippy 0 warnings / fmt clean
