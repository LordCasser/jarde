# Ref→Ref 槽复用基线（2026-10-09）

当前主线 `82090226`，实际候选 CLI9 SHA见 snapshot 内的manifest。八个完整Java类各由真 Corretto8与OpenJDK23编成Java8产物，分别带/不带调试信息。32条原输入与固定JADX输出全部完整重编、独立目录 `-Xverify:all` 运行一致；当前Jarde仅8条不同名LVT成功，16条无LVT和8条同名LVT均重编失败。

`baseline-patrol-20261009.tar.gz` 永久保存全部原始源码、jar/class、javap、runner、命令/双流/exit和manifest；archive hash与历史绝对路径映射在 [snapshot.json](snapshot.json)。其中名为candidate的腿是本次实现前的CLI9，不能误读为修复后结果。首次public类文件名错误的harness失败也保留，不计能力结果。归档原始字节未改写，临时路径将来缺失时按archive根映射读取。

root实际执行 [verify-baseline-root.py](verify-baseline-root.py)，核对1368个文件、432条命令、160次空classpath/sourcepath编译和112次独立验证运行，逐条核对jar成员、工具hash、源集和runtime stdout/stderr，见 [baseline-root-verification.json](baseline-root-verification.json)。另独立检查32个run方法均有两个astore_2，16个no-debug无LocalVariableTable，见 [baseline-slot-lines-root.json](baseline-slot-lines-root.json)。没有通过重建jar来假定旧zip一致。

因果锚：ArrayThenList将`new ArrayList`赋给`int[] local2`，StringThenBuilder将StringBuilder赋给String，两种失败均已有完整构造/调用正文，javac明确报类型与成员错误。源码含两个独立词法块，旧值在第二块前已死。此处javap可确认物理写入和类型来源；SSA唯一读取归属与完整CFG证明仍需产品测试及root验收，不将静态推论冒充动态dump。

本片按EM-20的SSA→源码变量原则取证。账本五项JADX测试不全是可编译Java：TestVariablesInLoop/Generic是smali，Generic明确禁重编，UsageWithLoops的第二测试实际引用EnhancedFor类，不能据测试名称宣称普通for已经验收。本片八类是受控正向家族，不能替代整个单元或完整LG验收。
