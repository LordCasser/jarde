## Context

缺口和八方法证据见 proposal.md。`decode::check_cast` 已提供池中的目标名称，`ExprKind::Cast`、`Type::Reference`、`spell_reference` 和一元/后缀优先级均存在。`render_value` 当前只擦除 `bridge_owns` 的 cast；instruction 分派会引用普通 cast，`renders_the_value_it_reads` 也只认 bridge。

这不是只增加一个 render 分支：普通 cast 会抛出 ClassCastException，必须跟随现有“生产者延期到消费位置；消费失败则保留来源”的协议。当前 `deferred_producers` 已处理调用、字段和数组效果，但尚未把普通 cast 作为需保留的效果。

## Goals / Non-Goals

**Goals:** 用现有表达式表示明确的检查，保持最终消费位置和检查来源；拒绝路径不丢效果。

**Non-Goals:** 不新增类型闭包、效果分析 pass 或转换简化。无消费位置的 cast 继续引用，不造临时变量把它拼成可编译语句。不把 cast 加入所有“纯值”白名单；区域、concat、初始化形状的拒绝须另测另议。

## Decisions

1. 复用现有 Cast。先读恰好一个逻辑栈值，递归始终传最终消费 `at`，使旧 local 值检查有效。bridge 已证明的擦除路径维持原逻辑与 derived 来源；其它 cast 以池目标经 `spell_reference` 得到 `Type::Reference`，创建带本指令 direct BCI 的 Cast，保留子节点来源。数组名复用现有描述符解析；不能仅把 `/` 替换为 `.`。缺目标、不可拼写或操作数失败沿用错误/引用。
2. instruction 分派在普通 cast 的结果确有现有可呈现 reader 时延期；无 reader 时引用 cast 和延期生产者。不能写 `(T)x;`，它不是合法 Java statement expression，也不能直接忽略检查。
3. `renders_the_value_it_reads` 让普通 cast 接入现有延期协议；随后 reader 的构造仍可能拒绝。`deferred_producers` 因而必须保留被延期普通 cast 的 BCI，并继续追溯其操作数；用现有去重、递归预算和最终消费点，不再建第二套效果表或全图遍历。bridge 已有来源归属不扩大为一般擦除许可。
4. 类型与 emitter 不因本项重新设计。现有 Cast 的 presented 类型是目标类型；后缀消费应已有 `((T)x).member` 分组，必须由实测确认。嵌套 Cast 保留每一层，不凭最终类型把前一层抹掉。若现有 emitter 有实际分组缺口，仅修本语法的分组。
5. 不引入外部库。池事实、描述符解析和 AST 已覆盖所需能力；增加通用反编译/类型库不会提供当前输入之外的证据，反而增加维护、许可与预算边界。测试只编译执行自写 fixture，无目标依赖下载。
6. 实施中的合法 `astore_1 → pop` 反例揭示现有静态 qualifier 识别过宽：`(String)x; pop; invokestatic ReferenceCastProbe.source` 会被拼成 `((String)x).source()`，String 并无该成员，且检查已单独引用。此轮不扩张静态 qualifier 规则；既有 discard 计划的 static-next 分支不认领普通、非 bridge 的 CheckCast 生产者。检查与 pop 保留引用，后续静态调用独立呈现一次；不发明继承判断或 owner cast。真正以 cast 为静态限定符的源码也暂留该缺口，待独立证明 qualifier 的类型/成员与唯一效果归属后处理。原版 fixture 的 astore 消费仍是正面恢复；只有内存 pop 副本是拒绝边界。

解析到目标类名是事实；是否能写出表达式是恢复判断；本项不启动 class loader/运行时选择/额外验证。受控执行对照只说明该 fixture，输出 verification 与 compilation 平面保持原值。语义依据为 [JVMS checkcast](https://docs.oracle.com/javase/specs/jvms/se23/html/jvms-6.html#jvms-6.5.checkcast) 与 [JLS Cast Expressions](https://docs.oracle.com/javase/specs/jls/se23/html/jls-15.html#jls-15.16)。

## Risks / Trade-offs

- 检查被当成可忽略值 → 把自写 fixture 的无用局部 store 同宽改为 pop，仅用于否定回归；检查和调用来源必须保留，不新增永久手工 class。
- 最终 consumer 不支持时只引用 consumer → 组合 cast、调用和尚未支持的指令，断言每个被延期检查的 BCI 仍在来源中，调用不重复。
- 嵌套顺序被简化 → 使用 `(Number)(Runnable)x`（Number 非 final），传 Integer 会在 Runnable 检查失败；若删掉中间检查则错误成功。
- null、错误数组类型、后缀解引用的异常混淆 → 运行时分别比较成功值/身份、null、ClassCastException、NullPointerException；不固定 JVM 实现相关的异常消息。
- 工作树同一 build 文件有其它语法修复 → fixture/test 可先独立准备，生产编辑必须等待 special change 移交；不同时写共享文件。
