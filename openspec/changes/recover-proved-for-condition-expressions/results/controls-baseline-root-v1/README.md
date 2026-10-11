# 控制基线真实验收

Root actual57命令/114raw，原JDK8/23各完整3类+Runner的19行行为恒同；JADX default/rename-none完整3生成类+原Runner双JDK四腿恒同。Jarde default/all各两JDK完整生成类原样保留，四合并编译失败；随后每目标生成完整类+其它两个原source+Runner的明确isolated legs中，Effects/Reads共八腿成功且全部19行恒同，Refusals四腿编译失败/no runtime。不能称三份生成类一起编译运行通过。

物理owner/name/descriptor/index/flags/Code/BCI/UTF8及default/all六对都核验。LeadingTickAndBreak既有fallback仅block leaders有来源，缺12BCI，见observations-root-v3.json/source_map_gaps；所有其它方法来源完整。这是明确baseline独立债务，不是本次最小ForHeader改动造成，也不混入修复。两正例条件共8方法当前均while；固定N2基线外while/内while。

原始preflight失败（local import re）、错误SHA256 digest假设的checker失败、忠实捕捉fallback来源缺口的checker失败均留invocations。Root分版修正后只接受基线观察，不是实现验收；BLAKE3只核shape/owner/location一致，class文件另作SHA256，不声称独立BLAKE3重算。接受凭据acceptance.json，原版archive-manifest-root-v1是copy时inventory；追加README/accept invocation见root-v2完整inventory。

冻结CLI v10是上一产品，4707输入与当前基线production恒同核准；应用新实现之后不得借它作为postfix验收。5GiB free/target1GiB一秒process-group守卫，采集未产生Cargo target。
