# 最终本地证据包

capture.tar.gz：4686292B，SHA256 ac025d0974ff090d038948e25d4e972a2ecd567a3a3cfd7bedc66ab441a055b0。138个payload加内嵌manifest共139成员，7份实际执行收据、78份收据绑定raw。已逐字节回读并独审；manifest.json为内嵌清单的原样副本。implementation.md（上级）解释实际范围。

包括旧public/CLI/in-flight反例、修复首验、最终focused、既有身份/flags/来源控制和最终CLI完整源码/双JVM/预算JSON实验；原状态和绝对路径保留，脚本仅作历史审计材料，不能直接重跑覆盖旧目录。package-execution与raw为实际打包记录，docs-strict为本地实施文档更新后的两版364/0。

最终focused457/0/1加专项21/0/0，均exact clean且target缺失；实际完整源码重编与四次Java8/23输出一致，同CLI三预算真实exit0/4/4。实验只覆盖原Runner固定输入。外置CLI/JDK/Python原生二进制不收入包，其actual pins在收据；baseline repro-v1独立保留。未来修复commit自身CI未执行，包成功不能替代该门。

后续集成说明（2026-10-15）：前述待CI是本地封存时状态，原raw/收据不改；自身产品aadf71的CI38703952159现已实际通过并独审，最终证据见../own-ci-v1与上级implementation.md。
