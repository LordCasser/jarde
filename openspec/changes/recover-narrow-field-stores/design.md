## Context

`field_write` 在真实 `putfield` / `putstatic` 的最终 BCI 渲染 receiver 与 stored value，再由共享 `field_value` 按 Fieldref 描述符调用 `meeting_position`。同一 `field_value` 也用于已经验证为字段写入的 synthetic accessor 调用。Java 赋值位置只允许合法 widening 或可表示的窄常量，不能自动表示 JVM 字段存储对任意 int 的 B/C/S 截断。当前拒绝路径 `fallback(vec![at])` 还只保留 put 指令，丢失前置值生产者来源；编译完整类时该拒绝方法可变成 `return;`。

`narrow-field-stores/` 的主 class 从普通 int 字段和真实写入编译；补丁仅改变 8 个 `field_info` 描述符及其匹配 `Fieldref/NameAndType` 为 B/C/S/Z，49 个 Code 哈希不变。补丁 SHA `713e821a6e182505d412ae2b8b01823e426cd436ac154073044fc19f1c6d2f10`，原 class/JVM 380 行有效；root 独立重放同字节、同 380 行，jarde 整类虽可编译运行却有 330 行不同，JADX 整类 javac 失败。Z 的 int 最低位语义独立处理。root 在补丁清单和 runner 输入阶段裁剪出 `core-bcs/`，仅六个 B/C/S 字段被改，patched SHA `da7bbcebd61dda7221f5169200a68b11fbe9200b8a4bb1f9d970bccc559157a5`；285 行有效，jarde 260 行不同，JADX javac 失败。永久 `p3-core/` 进一步缩为 1526 B/19 Code、267 行，patched SHA `d4797140eed55121de091518e7db4d9dcb5052d0c9a451ef6ded07e1589af8ef`；root 重放原/JVM 均有效，当前 jarde 整类可运行但 254 行不同，JADX 编译失败。普通窄字段局部/常量控制按每次写入后立即读取，不隐藏第一笔写入。

## Goals / Non-Goals

**Goals:** 在已证明的 B/C/S 字段存储指令处写出必要截断，保持一个字段写入、一次值求值、异常与来源。

**Non-Goals:** 不实现 Z 的一般 int 低位转换、未知引用转型、非本字段操作的隐式窄化、字段自增新结构或局部类型推理；不以扩大整类拒绝掩盖 no-op 语义差异。

## Decisions

1. 准入来自字段写入事实（真实 `putfield`/`putstatic`，或已验证的 synthetic accessor 写入）、字段 descriptor B/C/S、以及已呈现 byte/char/short/int 数值。仅在 `field_value` 这一消费入口用现有 Cast 包住需窄化的值；同型、合法 widening、可表示常量沿旧路径。普通 `meeting_position` 不改为全局允许 narrowing。
2. 保持 `field_write` 的最终 `at` 与 receiver/value 渲染顺序。Cast 不提前检查 null、不重复或移动生产者、不重读 receiver。静态字段和已确认的 accessor 写入共用该字段值合同；未确认 accessor 仍保持原拒绝，不凭名称猜。
3. Cast 与赋值仍锚定真实 put BCI，值与 receiver 的派生来源保留。失败分支用现有 `quoted_bcis(at)` 保留生产者（当前 `setByteProduced` 的 call@3/put@6 只剩 put@6），并沿现有节点预算、深度、取消规则停止；不造另一个来源收集器。
4. boolean 字段只用已有 boolean 证明，不能把任意 int 用 `!= 0` 写入：JVMS 对 Z 存储使用低位。该边界保留在完整 380 项，不迫使 B/C/S 正例类引用或改写 Z 语义。与数组写入、窄返回、显式转换分别闭合，避免一个通用“隐式转换”机制放宽不相干位置。
5. 不增加依赖。Fieldref、已呈现类型、Cast、FieldAssign 均已存在；外部求解器不会替代真实指令的存储语义，维护成本不相称。

依据：[JVMS `putfield`](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-6.html#jvms-6.5.putfield)、[`putstatic`](https://docs.oracle.com/javase/specs/jvms/se8/html/jvms-6.html#jvms-6.5.putstatic)。

## Risks / Trade-offs

- jarde 完整源码可编译但 no-op：必须逐项执行比对字段值、调用数与异常，不能以编译通过或带拒绝注释当恢复完成。
- 错误地把 null 检查提前会改变 `set*ProducedOn(null, …)` 的异常优先级；分别固定 producer 抛错、producer 正常后 NPE 与静态写入。
- 若只修普通字段而忽略共享 `field_value` 的 verified accessor，会造成相邻回归；测试已验证 accessor 合同，同时保持不成立 accessor 的拒绝。
