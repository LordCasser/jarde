## 1. 固定整类行为与限定边界

- [x] 1.1 将 `static-field-qualifier/` 自写 Java 8 源、runner 和真实 class 固化为永久 fixture，记录 javac 版本、class/Code/hash 与修前四行完整三方 javac/执行；原/JADX全同，Jarde `write` 的接收者次数由 1 变 2，不手改生成方法。Luna 固化在 `tests/fixtures/p3-popped-static-qualifier/`；root 原样独立重跑 `run_audit.py`，721B/7Code/哈希、源码逐字节重编、四行原/JADX同及Jarde差异、CLI哈希前后相同均重现。
- [x] 1.2 用合法源码或 JVM 验证的受控变体区分真正的表达式限定静态调用、静态字段读写、普通 `invoke;pop`、接口静态目标及目标属主不匹配；逐案记录是否可编译/可运行、BCI 和来源。同一 Code 可由三种源码写法生成时按语义等价验收；常量池目标换成其它类但仍能通过 JVM 验证时，记录原 JVM 与错误 Java 重绑定的差异，不把 opcode 邻接当成可表示性证明。Luna source-only 证据见 `static-field-qualifier/boundary/`；root 将整个目录复制到临时位置独立运行 `run_boundary.py`，冻结 CLI 哈希、4 组相同 Code、16 行原/JADX 对照、接口与两组 CP 补丁均重现。永久测试夹具仍归 1.1；失败消费者的来源边界归 2.3。

## 2. 单次消费与保守拒绝

- [x] 2.1 在既有 `discarded_evaluations` 中，对唯一由紧邻 `pop` 消费且可独立呈现的 `Invoke` 优先复用 `discards` 路径，不再同时成为下一静态调用限定符；其余已证明可限定的值仍由目标调用确切消费。使用原始 `write`、真正限定调用、普通弃值调用及接口静态调用测试确认生产者在完整正文中恰好一次，异常仍先于目标调用/字段写入。
- [x] 2.2 对仍需表达式限定的局部/字段值验证引用类型、Methodref 属主与 Java 成员选择关系；无法证明时保守拒绝，不输出异属主同名改绑或接口实例限定静态调用。以 1.2 的类型/属主/强转边界及原有静态调用测试验证。
- [x] 2.3 在消费者失败的引用路径通过现有延期生产者追溯附上限定 `pop` 所丢弃的值，保持真实生产者、pop、静态调用及最终消费者 BCI；默认/完整来源正文相同，来源/正文预算和取消仍可靠停止，以 source-map 与受限预算测试验证。

## 3. 整类执行与主代理验收

- [x] 3.1 用完整 Engine/CLI 原样生成并编译/执行 fixture 与 1.2 边界，原 class/JADX/Jarde 四行逐项相同且调用次数为 1；复跑普通静态调用、弃值调用、字段访问/写入与失败来源回归。
- [x] 3.2 root 独立审查归属、目标可表示性和拒绝来源，用重建冻结 CLI 重放完整类及独立边界；跑受影响 Rust/Java 测试、census/fingerprint、fmt、clippy 与 OpenSpec strict，分别记录既存门禁债务。结果见 `verification.md`。
