# 普通 switch 的非数值顺序穿透

`run_audit.py` 使用 SHA-256 `908c472560d6146354e1fd679c595e884afadf5ec7051727b51e79a1f95c6570` 的冻结 CLI，对同一个普通 Java 8 `SwitchFallthroughOrder` 做源码、JADX、jarde 整类 `javac --release 8` 与 `java -Xverify:all` 对照。class 394 B，SHA-256 `511fa56a325aeb1c2c7781e3fb0c86b6640cc4c3436df7cbb5c05ce0f5943ad0`。原 class/JADX 7 行完全一致；jarde 整类也能编译运行，但一处 `@bytecode`，其中两行错值：`case 9` 应得 91，却得 90；selector 调用仍一次。

`javap` 的 `lookupswitch` key 表按 1→BCI43、4→BCI49、9→BCI40 排列，实际代码顺序却是 `case 9` 的 BCI40 `iinc +90`，自然落入 `case 1` 的 BCI43 `iinc +1`。`region::switch_region` 按解码 key 的首次顺序建 group，又让各臂独立走到 join；它先认领 BCI43，再认领 BCI40，因此共享的 BCI43 被发现为重入，留作尾部引用。`build` 随 group 顺序生成 `SwitchArm`，`emit` 对每个非 return/throw 臂补 `break`。因此只给前臂加“穿透”标记并去掉 break 还不够：若继续按 key 顺序发射，`case 9` 位于 `case 1` 后面，会错误地穿透到后一个 default。

最小架构处理仍是既有 `present-proved-java-structure` 的 2c.4：在区域里证明前臂的唯一正常后继恰为另一个 case 的入口，在该入口停止前臂并让目标臂独占代码；在 AST/发射层按实际 case 入口 BCI 的代码顺序输出这组标签与臂，然后仅对已证实穿透的前臂不补 `break`。同一目标的多个 key 仍共用标签；非 case 入口的共享块继续拒绝。default 若是一个真实代码入口也应按其 BCI 排列；join-only 空臂及已有 return/throw 不应被误认成穿透。必须以原 class 逐行执行判定，而不能以 `javac` 成功或引用数变少判断。

此普通整数 switch 缺口与 `string-switch/variants/` 第二级 switch 的同类穿透互相验证，但不要求先把 hash/equals 映射折叠为 Java `switch(String)`。两份原始审计均保留为可重放证据。

`tableswitch/` 给出独立的密集键反例，root 原样运行其 `run_audit.py` 成功。390 B class（SHA-256 `f45d78320286e1912b30f4b01524b17695a1cef269a0204b18d3d7191ce16f84`）的表将 key 1/2 指向 BCI39、3 指向45、4 指向36，default 指向51；实际代码按 BCI36 的 case 4 `+40` 落入 BCI39 的共享 case 1/2 `+1`。原 class/JADX 的八行完全一致，jarde 整类编译运行成功，却把两项 4→41 写成 40，并留一处 BCI39 引用；其余六行和 selector 调用次数相同。由此确认 2c.4 需同时覆盖 `lookupswitch` 与 `tableswitch`，共享标签也要按目标入口而非 key 数值排列，无须新增第二套 switch 机制。
