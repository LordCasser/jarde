# DT-29 同包父字段：主线独立验收

root 审阅合入的 `d007e643`：只在既有直接父字段证书内新增相同包且 flags 精确为包可见 `0` 或 `ACC_PROTECTED` 的准入。原 public 路径保留，后续仍核对选中父类、唯一字段、CP owner/name/descriptor、实际 SSA 接收者和 BCI；输出显式父 owner cast，未改一般字段绑定与来源机制。

root 用主线 CLI（SHA-256 `7f8dafe1d928570cde099a10807a395ca209ea2e54f5479fd6295d99ec9dc92f`）独立重放 [脚本](replay.py)。原始、固定 JADX、Jarde 的完整 Java 8 类族均重编且 `java -Xverify:all` 输出 `true:true:false:false`；正例无 `@bytecode`。固定 `FieldCast$B.self` 的 BCI 7/12 均指向 A 的物理字段，Jarde 来源完整。private/static/final/volatile、错成员/层级/接收者、跨包 protected 负例拒绝；public-final 既有路径、private accessor 与预算控制保留。主线定向 `class_source` 1/1、`jarde` lib 148/148、`jarde-java` lib 233/233 通过，OpenSpec strict 和 diff check 通过。

root 重放还发现原证据脚本在 `javap` 标题和嵌套报告中写入临时路径、耗时；现已统一规范化并更新归档，以便重复回放不产生无关差异。`C/D.set`、泛型 `D`、根类 `run/bits` 仍不在本首片的成功声明内，由 [完整类族里程碑](../../../changes/recover-dt29-fieldcast-family/)统筹。
