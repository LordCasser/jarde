## Context

见 proposal 与 `shifts/root-core/`。root 从原审计分离了 `local` 中的异或和 `byteChar` 中的或，重新 javac 得到 719-byte ShiftCore，SHA-256 `db5a3053fb7b38ddf22cff1e9ec93048d41ab29e94d5809c0f1f611f8ac86bba`。这份 723 项基线不依赖位运算恢复。另 200 项 long 距离输入包含 `l2i`，当前该转换也未建模。

decode 只将 `0x60..=0x73` 映射为五种 ArithmeticOp，`0x78..=0x7d` 落到 Other。BinaryOp、binary_type 与 binary_binding 尚无移位；其中非算术默认 boolean 的旧分支不能用于新增运算。Builder 已有真实 SSA 操作数、最终消费点、生产者拒绝追溯与有界表达式恢复。后续实施以已验收的共享延期值保存为基础，不另建效果判断。

## Goals / Non-Goals

**Goals:** 用现有 Binary 表达六个移位事实，显式保持结果宽度、类型与 Java 分组，在当前可接受区域完成真正可编译的恢复。

**Non-Goals:** 不补 `l2i` 或其它转换、不识别位运算美化、不新增 `<<=` 等赋值节点，不扩大 phi/guard/循环准入，也不实现常量求值器。原 ShiftAudit 的跨位运算组合待两项各自完成后验收。

## Decisions

1. 读取六个 opcode 的同源事实，用三种移位方向及既有 Int/Long 形状核对操作数。复用当前事实枚举的惯例，不把操作伪装成加乘，不复制一套 frame/SSA 类型。AST 只增加 BinaryOp 的左移、带符号右移、无符号右移三个成员。
2. 类型是左右分别作一元整数提升，结果取左侧类型。byte/short/char/int 左值呈现 int，long 左值呈现 long；右值的类型不提高结果宽度。构造时还要核对 JVM 的右操作数为 Int 形状和源码确为整数，不能把 descriptor 为 boolean 的同形值写成 Java 移位。浮点、boolean、未知类型不靠 presenting 标签强行准入。沿用现有类型小查询，不新建 promotion framework。
3. 直接写 Java 移位，保留 Java 与 JVM 已共有的距离低位规则。不要生成额外 `& 31` 或 `& 63`，不要把 `>>>` 改成除法或把移位重写为乘法；这些改写增加节点，也容易改变宽度及负数语义。基本范围的距离生产者只使用当前可恢复 int 型值。long 距离的 `l2i` 留给独立数值转换任务。
4. 在统一 emitter 的绑定强度中将移位放在加减与关系之间，仍用同一左右分组规则保留右侧同优先级嵌套。统一调整 PRIMARY/UNARY 与其它二元级别，不能局部加括号字符串或留下强度相撞。位运算若已落地，合并到同一个完整优先级表。
5. render_value 读取两个真实 stack operands，并向下传递最终 at 与深度。延期 producer 的 reader 判断、失败追溯与正常消费保持一致；依赖共享求值位置证明及保存机制，不能因为 shift 本身不抛异常便将操作数调用/字段越过独立语句。已有循环纯值名单如需接入，只增加此操作，不接纳新的调用或存储。
6. 节点 origin 取实际移位 BCI，操作数、消费者与 member 沿用现有路径；预算、取消、深度和 source-map commit/replay 继续统一收费。没有新的全图扫描或持久状态，拒绝时保留已恢复前缀及剩余生产者引用。
7. 不引入库。已有 reader 已解码 opcode，现有表达式/类型/发射设施能闭合问题；外部库不能提供比同源 SSA 更好的值身份，增加维护及许可成本无收益。自写 javac/java 对照是验收工具，不改变产品 parse、dialect、runtime、verification 或 source-compilation 平面的承诺。

本地 JADX 1.5.6 的 `SimplifyVisitor.isArithWideUpCast` 特别保留算术操作数上使寄存器宽度增加的转换；源码注释用 `(long) i << 32` 说明删掉这层 cast 会让 Java 改选 int 移位指令，从而改变结果。这个观察值得沿用，但 Jarde 的证明应直接从 `ishl`/`lshl` 等原 opcode 与左操作数 SSA 类型确定 int/long 结果，必要的已有 Cast 留在表达式树中；不能仅凭通用二元提升或“两个操作数谁更宽”来决定宽度，也不能在打印阶段补猜测性的 cast。

完成后的[求值位置反例](../../evidence/java-syntax-2026-09-24/shift-effect-order/analysis.md)进一步说明不能照搬 JADX 的表达式内联。真实 `ishl` 与两侧 `step()` 先于 `getstatic events`，JADX 1.5.6 却把字段读写进包含移位的加法左侧；生成源码能编译，但三行执行都与原 class 不同。Jarde 沿既有 SSA 最终消费点和保存语句输出局部结果，三行保持一致。这里不需要新增 shift 专用求值机制。

语义依据：[JLS 15.19](https://docs.oracle.com/javase/specs/jls/se23/html/jls-15.html#jls-15.19)、[JVMS ishl](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.ishl)。

## Risks / Trade-offs

- 套用 binary promotion 导致结果变宽 → 核对左值独立提升、byte/char 与 long 的完整重编译输出，并保留 long 距离转换为单独边界。
- 同形 boolean 被输出为整数 → 合法 descriptor 变体经真实 JVM 验证后固定拒绝测试，不用非法 class 测“安全”。
- 右侧嵌套丢括号或多级绑定冲突 → 直接测试嵌套移位、加减、关系及已支持位运算的共用发射器；实际执行不以字符串存在代替。
- 新 consumer 扩大延期错序 → 左右调用、抛错、旧局部及独立语句对照依赖同一求值位置契约，不写 shift 特有临时缓存。
- 为等待相邻实现而扩大本项 → 串行接入已验收基础；基本 ShiftCore 单独通过即可证明移位闭环，组合覆盖另行列数。
