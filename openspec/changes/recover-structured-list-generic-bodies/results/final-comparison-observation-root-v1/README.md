# 最终完整类对照的独立验收

root 实际执行 v2 checker，因 Runner 编译使用 capture 的副本路径、校验器却绑定原 manifest 路径而退出 1；v3 路径修复后又因自身 pins schema 未同步退出 1。v4 仅补齐版本绑定，经 root 审查后实际退出 0。三次调用和全部 stdout/stderr 保留，前两次未生成 summary，不改写为通过。

六家族 fixed/String/Integer/controls/inner-option/null-add 各 debug/none，共 12 份双 JDK 原/JADX/Jarde 对照。实际 352 命令、704 原始流、41 个独立 oracle labels；80 组完整生成类编译运行（Jarde 40、JADX 40）与冻结原类输出逐字相同，包含值、集合状态、异常和效果。24 组合并控制编译失败，没有 runtime，handler 每模式 4 处来源缺口保留。没有手修生成源码。

Jarde 正例恢复确切 List<String/Integer> 和合法 add；真实 checkcast、remove(Object) 保持。内层 Integer 探针使用两份独立 new Integer(5)，新增值确为 inner 对象、不是 outer 对象；String/Integer add(null) 均保留异常与集合状态。完整生成 source、传给 javac 的 target/Runner 副本、生成 class、原始报告和 source map 都核 exact bytes/SHA/物理身份，default/all 正文与物理 map 一致。

新 CLI 的 late-stop 独立观察也已接受：limit=32531，used=25299，requested=8112，在最终 List 再证明停止，exit 4/Partial/AnalysisSteps；raw 方法正文和 source map 与完整请求原始 report 相等，没有泛型 overlay。第一次记录器使用默认仓内路径假设而在 CLI 结束后异常，未记录 CLI exit，失败原样保留；修正仅记录器路径后 fresh probe 实际完成。

本目录只验收整类比较及停止观察。summary 的 product_acceptance=false、task1_2_complete=false 正确保留：反例 SSA 归因、完整工作区/自身 CI/最终主线交付另行验收。当前 task 3.1 的完整类对照判据已满足，不能据此宣布整个 CF11 或 71/612 追平。

压缩档保留原外置目录名，receipt 内绝对路径未改写；当前 CLI root-v2 SHA bf9757a42d70cb01909466595baa4ecfdbfed7cd585527e008944290ce7736b9，focused v9 与完整 4822 文件来源绑定。需要重跑校验时将档案解至 /private/tmp，并保留此前冻结依赖与 CLI。archive-pins.json 核本目录各项字节身份。
