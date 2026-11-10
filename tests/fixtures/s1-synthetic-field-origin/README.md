# R07 synthetic 字段来源回归

`supplementary.jar` 来自已有 patrol 的 `N1.class`、`N1$Inner.class`、`N1$Stat.class`。只替换 Inner 的一个 CONSTANT_Utf8：`this$0` → `this$𐐀`（MUTF `7468697324eda081edb080`），并更新该项 u16 长度；CP索引、Code/BCI和其余字节保持。没有新增 loose class 或修改 reader 人口断言。

生成脚本、源文件与逐项SHA-256见 [R07证据](../../../openspec/changes/restructure-demand-driven-recovery/results/s1-synthetic-field-origin-v1/README.md)及其中 `make-inputs.py` / `input-manifest.json`。该夹具是定向名称变体，不是独立holdout。
