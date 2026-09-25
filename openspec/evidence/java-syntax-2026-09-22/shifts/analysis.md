# Shift syntax audit

2026-09-23，用当前 `target/debug/jarde-cli` 做确定性、临时语料审计。可重放脚本是
`run_audit.py`；它在 `/tmp/jarde-shifts-audit` 下以 `javac --release 8 -g:none` 编译，先用
`java -Xverify:all` 执行原 class，再对同一 class 做完整 JADX 输出的编译与执行，最后用
当前 CLI 的完整 `class-source` 输出编译。源码、javap、CLI stderr、三份编译日志、运行输出和
统一 diff 均保存在本目录；没有修改生产代码、永久 fixture 或 Cargo 产物。

## 结果

`ShiftAudit` 是基本正面组，覆盖 int/long 的 `<<`、`>>`、`>>>`，负数、MIN/MAX、负/超宽
距离，嵌套优先级，局部复制，分支消费，byte/char 提升，以及左右调用和抛错顺序。原 class
720 bytes，SHA-256 `f45306f5b358d02314bfd8ed42255a841f78885cf4cb261f9dfee9d86ec257c1`，
`-Xverify:all` 输出 723 行。JADX 完整类编译、验证、运行成功，输出与原 class 逐行相同
（`jadx_diff_lines=0`）。当前 CLI 返回成功但产生 31 个 `@bytecode`，恢复类的 javac
失败（11 个缺少返回语句）；因此没有伪造 jarde 执行结果，`basic-jarde-diff.txt` 明确记
录未运行。

`ShiftDistanceBoundary` 独立保存 int 左值和 long 距离的组合，源码显式 `(int) distance`，
因此 javac 生成的实际路径包含 `l2i` 后的 `ishl`/`ishr`/`iushr`，也保留局部消费。原
class 364 bytes，SHA-256 `4038c308a4c9554e582089aa03ad44e361f66875e28ab7732d21dc70826148f1`，
`-Xverify:all` 输出 200 行。JADX 完整类编译、验证、运行成功且逐行相同；当前 CLI 产生
13 个 `@bytecode`，恢复类 javac 失败（4 个缺少返回语句），没有执行 jarde 结果。

两组记录的 CLI SHA-256 都是
`c050b502ba3fc23f33937ac607b1da7b51447e02e1a6d41b1d060c2b93b5b8d9`，细节见
`summary.json`。完整输入和固定边界值由 `ShiftBasicRunner.java` 与
`ShiftDistanceRunner.java` 保存，第二组没有混入基本 shift 输出。

## 当前代码事实与缺口位置

- `crates/jarde-java/src/decode.rs:147-150` 将 `0x60..=0x73` 交给 `arithmetic`，而
  `decode.rs:240-249` 的 `arithmetic` 只覆盖加、减、乘、除、余；shift 的 `0x78..=0x7d`
  因此落到 `Operation::Other`（`decode.rs:186` 的默认分支）。`facts.rs:243-249` 的
  `ArithmeticOp` 也只有这五项，`facts.rs:452-463` 的 `Operation` 没有 shift 事实。
- `ast.rs:74-91` 的 `BinaryOp` 当前只有五种算术和五种关系；`ast.rs:465-479` 只对五种
  算术做数值提升，其余操作直接按 boolean 类型返回。这个现状不能把 shift 借用成比较
  或普通算术；shift 需要明确的整型结果与右操作数距离规则。
- `ast.rs:481-499` 已有 byte/short/char/int 到 int、long 保留为 long 的提升事实；基本组的
  byte/char 输出和独立 `l2i` 组说明，恢复时必须分别处理左值结果类型与 JVM 对距离操作数
  的 int 化，不能把所有 int-shaped 值按一个 Java 源码类型输出。
- `emit.rs:1081-1087` 的优先级表只列乘除余、加减、关系、相等；Java shift 应插在加减和
  关系之间。基本组的 `nested`、`local` 和 `branch` 只作为现有 emitter 形状的边界证据，
  没有把未实现的 shift 当成已恢复。

该审计只保存代码事实和可重放结果，没有引入新的 Operation、Binary 或类型框架。

root 复核补充：`0x7e..=0x83` 是另案位运算，不属于六个 shift opcode。`l2i` 当前也
没有独立 Operation，仍落到 Other；不能因已有源码 Cast 节点就宣称 JVM 数值转换已经
恢复。因此基本移位的最小闭环可以只扩展既有 Binary 与忠实操作事实，long 距离转换组
继续作为独立组合边界，不能把那 200 项包含在六个移位指令实施后的覆盖承诺中。

## Root 独立隔离与拒绝边界

进一步审读发现原 basic 的 local 含 xor、byteChar 含 or，因此原 723 项是移位加位运算的组合输入，不能仅凭六个 shift 实现承诺整类通过。root 将这两个表达式在源码中改为现有加法，重新编译成独立 ShiftCore（并非修改反编译输出）；`root-core/` 保存 719-byte、SHA-256 `db5a3053fb7b38ddf22cff1e9ec93048d41ab29e94d5809c0f1f611f8ac86bba` 输入的完整证据。新 723 项原 class/JADX 一致，CLI 7527b03a 前后hash一致，29引用、实际完整javac失败。这份是最小移位 change 的基础。

`boolean-boundaries/` 精确修改唯一的UTF8方法descriptor `(II)I` 为 `(ZI)I` 或 `(IZ)I`，未改Code，随后直接用patched class编译runner。两份合法JVM输入以 -Xverify:all 分别执行14和10项，JADX完整类也通过并相等；jarde各2引用、完整javac失败。JADX使用boolean到0/1的显式条件表达，说明它们理论上可以扩展恢复；当前小任务只闭合直接整数移位，不增加boolean数值化规则。后续实现必须明确拒绝这两种直接boolean移位，不能因为frame同为Int而生成非法Java。各class hash、源码、补丁脚本和完整日志均已保存。

`recover-shift-expressions` 的四份规划已strict通过，先实施已确认的错值修复，再串行接入本项。
