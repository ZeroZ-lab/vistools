# Acceptance Commands — 摄影计量 P0

> 手动验收时按这个顺序执行，避免遗漏模式和局部区域检查。

## 0. 前置验证

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt -- --check
```

## 1. 固定 fixture 冒烟

```bash
cargo run -- histogram fixtures/e2e/landscape_large.jpg --rgb
cargo run -- zone-map fixtures/e2e/landscape_large.jpg
cargo run -- exposure fixtures/e2e/landscape_large.jpg --mode evaluative
cargo run -- exposure fixtures/e2e/landscape_large.jpg --mode highlight-weighted
```

```bash
cargo run -- histogram fixtures/e2e/portrait_tall.jpg --rgb
cargo run -- zone-map fixtures/e2e/portrait_tall.jpg
cargo run -- exposure fixtures/e2e/portrait_tall.jpg --mode evaluative
```

## 2. 局部区域检查

```bash
cargo run -- histogram fixtures/e2e/screenshot_like.jpg --rgb --rect 0,0,800,600
cargo run -- zone-map fixtures/e2e/screenshot_like.jpg --rect 0,0,800,600
cargo run -- exposure fixtures/e2e/screenshot_like.jpg --mode spot --x 400 --y 300
```

```bash
cargo run -- histogram fixtures/e2e/urban_square.jpg --rgb --rect 500,500,600,600
cargo run -- zone-map fixtures/e2e/urban_square.jpg --rect 500,500,600,600
cargo run -- exposure fixtures/e2e/urban_square.jpg --mode center-weighted
```

## 3. 真实摄影样本模板

把 `<PHOTO>` 替换为真实照片路径：

```bash
cargo run -- histogram <PHOTO> --rgb
cargo run -- zone-map <PHOTO>
cargo run -- exposure <PHOTO> --mode evaluative
cargo run -- exposure <PHOTO> --mode highlight-weighted
```

如果需要测局部主体：

```bash
cargo run -- histogram <PHOTO> --rgb --rect <X>,<Y>,<W>,<H>
cargo run -- zone-map <PHOTO> --rect <X>,<Y>,<W>,<H>
cargo run -- exposure <PHOTO> --mode spot --x <PX> --y <PY>
```

## 4. 建议记录顺序

每张照片建议按下面顺序记到 [acceptance-log-template.md](/Users/zhengjianqiao/workspace/vistools/docs/features/photography-metering/acceptance-log-template.md:1)：

1. 先写人工标签
2. 再看 `exposure evaluative`
3. 再比较 `highlight-weighted`
4. 最后记录 `histogram --rgb` 和 `zone-map` 是否提供了增量信息

## 5. 最终汇总

验收完成后，把以下 3 个数字回填到汇总表：

- `exposure.assessment` 一致率
- `histogram --rgb` 增量占比
- `zone-map` 异常样本数

再按 [acceptance.md](/Users/zhengjianqiao/workspace/vistools/docs/features/photography-metering/acceptance.md:103) 的门槛给出 `pass / partial / fail`。
