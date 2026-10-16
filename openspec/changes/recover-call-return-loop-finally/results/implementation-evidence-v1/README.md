# 最终实现、完整源码及父级重筛证据

capture.tar.gz 为 10,324,443B，SHA-256 e52c677bdb41266e645ab3ae05bcedd65147cebd64c6261005099baa66c237df；369 payload 加一份嵌入 package_manifest.json，共370个安全唯一普通文件，外部 manifest逐项校验。它保留失败/中间/最终 native、原始流、源码快照、完整类重编/运行、父screen与独审、实际4936行source/blob/mode/CLI/product-head绑定。

最终native前27命令110 passed/0 failed/8 ignored；后9命令补强同名public2/CLI3断言，不能叠加。最终完整类10命令、父screen24命令全0。两独立完整类各1706B原样重编、四次未改driver rebuilt-only运行精确217B原始oracle；父6类/12main也匹配，但FX finContinue/finBreak handler45仍fallback。

first-proof正例失败、公共v1前置检查失败、公共v2发现entry缺口都按实际状态保留。包内绝对路径是历史provenance，搬迁后须按manifest映射；完整checkout、冻结CLI/JDK/Rust工具链均不打包，外部pin明确记录，不能据此声称脱离依赖可直接运行。原始规划/控制/actual Facts另见planning-baseline-v1；本包不替代产品自身CI。
