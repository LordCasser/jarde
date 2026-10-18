# 已冻结的 ForHeader 条件输入

五份 class 是 root 实际 JDK23 --release8 -proc:none -g 编译字节，source 原样保留；对照 Runner 与完整 raw 在 recover-proved-for-condition-expressions/results/controls-baseline-root-v1 和固定 cf11-testnestedloops2-baseline-root-v1。

Effects/Reads 正例和 AssignedCallBound 的 baseline 行为及来源完整。旧版 LeadingTickAndBreak 的 fallback 不可编译，且缺12个physical来源；该历史拒绝记录保留在上述 baseline 中。`recover-statementful-loop-headers` 现以既有 Endless Loop、If.test_effects 与 LoopBreak 恢复该完整方法，不投影成 counted-for。最终 CLI 对双 javac 输入、default/all 的五个方法均恢复 Structured Java，包含所有物理 BCI；生成整类源码原样重编的16次 Jarde 双 JVM 运行、192条返回/计数观察与原类一致，JADX另8次运行96条亦一致。完整51命令/102 raw与独立核验见该 change 的 `results/legacy-leading-tick-final-matrix-root-v1`。固定 NestedLoops2 的 raw List 签名投影独立于循环形态。

```json
[
  {
    "class": "ForConditionEffects",
    "class_sha256": "9ef5bc9d7701575e685e1469a41a17d7798cf932d3230a15b276e7d0c69885df",
    "source_sha256": "1955b4a9aaf6787a5d16f3c927dce98888d3322389118f137081a959529c81e1",
    "original_compile_leg": "javac23 --release 8 -proc:none -g",
    "jdk8_class_bytes_equal": false
  },
  {
    "class": "ForConditionReads",
    "class_sha256": "7664451909614ce76d089ede2cac5b7529c939fc565d4aede24f419825c7925e",
    "source_sha256": "4c30b5e9435e0dfc260a6849cc2f28b5495e7b5a7847615ab59f0a9870be18a5",
    "original_compile_leg": "javac23 --release 8 -proc:none -g",
    "jdk8_class_bytes_equal": false
  },
  {
    "class": "ForConditionRefusals",
    "class_sha256": "9afb4eeeb1558845789c2f4b929d3f6adcac237edb3484a11d4a0b68ddc1619b",
    "source_sha256": "e49e456fd8e128e8cddc108f3f2a5db1e863b1450a3fb8e9523ff3e19886a454",
    "original_compile_leg": "javac23 --release 8 -proc:none -g",
    "jdk8_class_bytes_equal": false
  },
  {
    "class": "UpstreamNestedLoops2",
    "class_sha256": "1b11e7ac235ff48798edb6bd1f47c02829592bdbcf7f4af0066fec278d095547",
    "source_sha256": "ee79a454a2e92da4f50dad00a5e6bc1d33388bc00f028d0c5213f0befdca0cf5",
    "original_compile_leg": "javac23 --release 8 -proc:none -g",
    "jdk8_class_bytes_equal": false
  },
{
    "class": "ForConditionNumeric",
    "class_sha256": "f5a95d227993e466ebb71e07df0697ac1c34518b2da58f5ae141cd5e418bdaab",
    "source_sha256": "5cd7a82fa49484a4706270ab4bc63364c81a4725287b9b264b35992d5a641396",
    "original_compile_leg": "javac23 --release 8 -proc:none -g",
    "jdk8_class_bytes_equal": false
  }
]
```

Numeric 对照另见 results/numeric-baseline-root-v1：long/float/double array 条件与 NaN、null、越界和 long 极值共十五项真实 oracle。五类人口增5 class/24 Code/0 handler/41 branch target/0 jsr，由root实际javap测量。
