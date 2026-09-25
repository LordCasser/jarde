## Context

root 的证据均为普通 javac 输入，无字节码补丁：

| 输入 | 原 class/JADX | 当前 jarde |
| --- | --- | --- |
| 默认包 `arg0`，359 bytes，SHA `6df3f4ccd7ce4fcbaca22c4a38f38cb0f637ce01d5d7b0df0c4ececf98846363` | 6 行一致 | 零引用、javac 通过，字段读写 4 行错误；本类静态调用省略 owner 是正确对照 |
| `external/ShadowExternal`，340 bytes，SHA `ab42b65ce6bdec23cf980037232d946203c2459f07858fb1804cfb6935992445` | 6 行一致 | 零引用、javac 通过，静态调用/读/写 6 行均错误 |
| `package-prefix/ShadowMath`，debug 名 java，468 bytes，SHA `d775ee2998d9e6d18cb0a2225ce186c341fb4fa6e04d784af10a53439bc22a0c` | 3 行一致 | 零引用，`java.lang.Math.abs(value)` 被局部 java 遮住，javac 失败 |

三份基线 CLI SHA 均为 `7527b03abc1e487121d204672b52043ca5f3aa1148c6d65b136e36f2bbe1f4aa`。AST 已区分 Path 和 Local，但 Java 源码中的名称分类仍会将 `arg0.value` 的首段认成变量。给 Path 更强的内部类型标签不能修正已经发出的文本。

NameTable 现有 reserved 集合已经为简单 blank final 字段预占名称，并同时供普通名称与 free_name 使用。report 在字段计划完成后构建 NameTable，尚未加入静态 owner 约束；调用与字段所需 owner 已在同源 Operation 和字段计划中。无需改 SSA 身份或引入类型解析器。

## Goals / Non-Goals

**Goals:** 防止可由本方法命名决定的变量遮住普通静态调用/字段的类型限定符，保持成员目标与既有命名证据。

**Non-Goals:** 不任意重命名真实字段/class，不修包名被类成员遮住的声明问题，不做 import 简化、内部类重组或跨类层级解析。方法引用限定符另案；不能为收集其 owner 重跑 lambda::plan 或新增一套 bootstrap 解释。本项不调整 pop-qualified 静态调用的原有 receiver 求值语义。

## Decisions

1. 在现有命名准备中增加一次受预算限制的约束收集，输入为本方法已经读过的 Operation、字段 claim 与声明事实。只看实际 body 引用且现有静态呈现路径可能使用的 owner，不能扫描整类所有未使用常量池项来大范围改名；不新增恢复 pipeline 阶段、cache 或 resolver。
2. 限定符使用现有 spell_reference 的唯一拼写，从可拼写的类型路径中取得首段：默认包类型 `arg0` 预占 arg0，`java.lang.Math` 预占 java。不是预占整串带点名字，也不是把末段 Math 当成唯一冲突点。未知或不可拼写 owner 继续沿原拒绝规则处理，不能为它发明安全类型名。
3. 对静态字段读写消费已有 field claim 的 owner 与简单 final 写入证明；简单 final 名称约束仍保留。普通 invokestatic 的 owner 来自原 CallTarget，保持当前本类省略限定符与显式表达式 qualifier 的决策。收集可能出现的静态 owner 属于保守命名约束，不授权新的调用恢复；不要为了精确排除无需限定的 pop 情形重新分析其效果。
4. 将约束合入既有 reserved 输入，所有局部、参数、debug 分段名与非槽名称共用原候选过程。每个新后缀都再次避开全部约束，覆盖同时存在 `arg0` 与 `arg0_2` 类型名的情况。保留 raw debug 证据和原 Collision 别名契约；合成名不冒充原局部槽或 debug 名。无相关冲突的方法保持原名。
5. 不在 AST 生成后字符串替换或全树重命名，不删除字段 owner，不把静态成员访问改成通过某个碰巧兼容的实例。也不普遍加入 `((Owner)null)` 伪接收者来逃避命名问题：现有 NameTable 可以解决本范围，额外表达式会增加来源和求值证明负担。
6. 收集遍历、集合条目及名称碰撞尝试沿用既有预算和取消通道；检查实现已有名字循环的计费边界，只为本项新增工作补齐，不能引入无预算 owner/前缀枚举。默认和完整证据共享同一名称决策，source-map replay 不再次改名；既有静态访问原始 BCI/CP/member 仍可追溯。
7. 不引入外部库。局部命名是当前恢复层可直接决定的事实，已有集合及类型拼写足够；外部类路径解析增加维护、许可和预算成本且不能解决源码中的词法遮蔽。原样 javac/java 是自写输入验收，不改变产品解析、dialect、运行时、verification 或可编译项目承诺。

依据：[JLS 6.5.2](https://docs.oracle.com/javase/specs/jls/se23/html/jls-6.html#jls-6.5.2)。这里修正的是 Java 名称重新分类对输出绑定的影响，不改变字节码的字段/方法解析事实。

## Risks / Trade-offs

- 只测试 javac 成功漏掉错目标 → 原样完整类对照调用返回、两个不同字段的值，并覆盖 null 参数；静态访问不因 null 自动失败。
- 仅避开类型末段，包首段仍被局部遮住 → 保留 debug java 正例和默认包类型正例。
- 第一个候选不冲突但追加后缀撞到另一 owner → 同时预占 arg0/arg0_2，复跑最终 shared free_name 与合成绑定命名。
- 扫描全部 CP 导致无关别名漂移 → 用未被方法引用的冲突 owner 作对照，收集范围限于当前 body 与现有字段决定。
- 真实成员或方法引用碰撞扩大范围 → 保留独立证据，不重命名外部 API，也不借本项建立类级命名系统。
