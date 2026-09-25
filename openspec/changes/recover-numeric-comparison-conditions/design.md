## Context

证据见 `../../evidence/java-syntax-2026-09-22/comparisons/analysis.md`。既有 `CompareOp` 描述条件分支，不是 `lcmp/fcmp*/dcmp*` 产生的 -1/0/1 值；后者当前为 `Operation::Other`。`condition` 已先决定 taken/fall-through 的整数谓词，随后呈现操作数；`Binary` 和 `Not` 能表达所需 Java 条件。SSA 已提供定义、使用点、同块指令序列，无需再建消费图。

## Goals / Non-Goals

**Goals:** 闭合“数值比较结果直接供给条件分支”的最小组合；保留浮点无序值的语义与已有来源/消费协议。

**Non-Goals:** 不赋予比较结果一般 Java 值拼写；不处理存入 local、dup、多使用者、跨块或 phi 后消费。布尔返回汇合与含调用循环仍按原区域准入；不把新事实无差别放入 concat、构造器、guard 的所有白名单。

## Decisions

1. 增加一个忠实操作事实，区分五种原 opcode（long、float 两偏置、double 两偏置）。它是解码结果，不是恢复 verdict；不增加 AST 或 pass。用现有枚举中的一个新分支及小枚举即可，禁止用算术减法冒充比较结果，也不借用条件分支的 CompareOp 隐去 NaN 事实。
2. 组合证据限定为：比较读恰好两个逻辑栈值、产生一个栈值；该值的 SSA 唯一使用者是同一基本块紧接的六种零分支之一。无 phi、dup、local 传递或介入指令。借用现有 SSA uses/定义与相邻指令检查，不增加全图扫描、消费 registry 或缓存。instruction 是否延期、reader 是否承接操作数、condition 是否组合使用同一边界判断，避免各自放宽。
3. 先由既有分支极性得到对整数结果的实际谓词，再按下表转换。两操作数均在最终 branch BCI 递归呈现，不能缩回比较 BCI 绕过旧 local 检查。用现有 Binary/Not，一次呈现每个操作数，保持从左到右；不新增 isNaN 调用，不重复执行操作数。

| 结果谓词 | lcmp | fcmpg/dcmpg | fcmpl/dcmpl |
| --- | --- | --- | --- |
| == 0 | a == b | a == b | a == b |
| != 0 | a != b | a != b | a != b |
| < 0 | a < b | a < b | !(a >= b) |
| <= 0 | a <= b | a <= b | !(a > b) |
| > 0 | a > b | !(a <= b) | a > b |
| >= 0 | a >= b | !(a < b) | a >= b |

4. 节点保留比较 BCI、分支 BCI 和两个操作数来源。比较本身只有已证明被该分支承接时才不写独立引用；其它消费者仍明确 quote。后续条件构造失败时，现有 region quote / deferred producer 追溯必须涵盖比较与嵌套调用。允许在已存在的数值循环 test 准入里加入纯比较事实，但不放开调用、store 或 cast 等其它操作；直接调用操作数的 if 必须保持执行次数，原本拒绝的循环继续拒绝。
5. 不新增库。映射只依赖 JVM 五个操作与 Java 比较/取反的有限语义；现有读取、SSA、AST 和 emitter 足够。引入第三方反编译或求解库不提供缺失输入事实，也不值得增加维护和许可负担。解析事实与呈现判断仍分开；不把 fixture 的受控编译执行升级为产品的 verification 或 compilation 结论，不读取外部类或执行任意目标程序。

语义依据为 [JVMS 比较指令](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.dcmp_op) 与 [JLS 数值关系](https://docs.oracle.com/javase/specs/jls/se23/html/jls-15.html#jls-15.20.1)。分析中的 1,944 项映射枚举已通过，但实现仍须独立执行原 class 与恢复文本对照。

## Risks / Trade-offs

- NaN 极性被普通布尔反转误简化 → 对五种 opcode、六个零谓词、两种分支方向做表驱动验证；执行至少两侧 NaN、正负零、无穷及有限值。
- long 关系用差值导致溢出，或 Java compare helper 改变零/NaN 行为 → 输出只用 Binary/Not；long MIN/MAX 直接对照。
- 为恢复一个条件吞掉其它消费者 → 固定唯一、相邻、同块边界；合法 patched class 验证非条件消费/多使用者仍引用且效果有来源。
- 表面条件正确但求值重复或调序 → 左右调用累加不同 trace、各自抛不同异常，与原程序逐项比较；无辅助调用展开。
- 失败构造漏来源 → 对拒绝操作数、旧 local 值及生产者引用测试；不得为通过测试搬运赋值或重新命名旧值。
- 与引用转换/重载共享 build 文件 → fixture 可独立准备，生产必须由 root 按所有权串行移交；比较不抢占正在实施的 cast。
