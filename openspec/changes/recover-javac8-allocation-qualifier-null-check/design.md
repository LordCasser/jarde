# Design：让分配限定符形在真 javac 8 下恢复

## 决策依据（root 已实测，全部可在 [getClass 片证据](../../evidence/java-syntax-2026-10-04/recover-javac8-getclass-null-check-idiom/) 复现）

**字节码事实（`N1.main` 的 `new N1().new Inner(3).total()`，双腿 javap 对照）**：

```text
        javac 23 --release 8                真 javac 8 (Corretto 1.8.0_432)
 12: new           N1$Inner            12: new           N1$Inner
 15: dup                               15: dup
 16: new           N1        ← 外层分配 16: new           N1        ← 外层分配
 19: dup                               19: dup
 20: invokespecial N1.<init>           20: invokespecial N1.<init>
                                       23: dup            ┐ javac 8 插入的被丢弃
                                       24: invokevirtual  │ null-check 三元组
                                            Object.getClass│ （外层实例的读者）
                                       27: pop            ┘
 23: iconst_3                          28: iconst_3
 24: invokespecial N1$Inner.<init>     29: invokespecial N1$Inner.<init>
```

即：javac 23 对新分配的外层实例**不做任何 null 检查**（可证非空），而真 javac 8 **无条件**插入 `dup; getClass; pop`。这条 `dup(23)` 读取外层实例（BCI 20 的构造器产物），是外层分配站点的一个**额外读者**。

**第一道门（root 实测钉死）**：`init.rs` 的 `new@1` 实例读者门。`verify_new` 计算 `produced_by = [head=16, dup=19, at=20]`（外层站点的三条自有指令，`init.rs:485`），再 `outside_readers(ssa, &produced_by)`（681）收集**站点外**读取该实例的指令——`dup(23)` 不在 `produced_by` 内，故被计为读者；`renders_its_reads`（1331-1347）对 `Operation::Duplicate` 返回 `false`（落 `_ => false`），故 `written.is_empty()`（687）→ 返回 `init.rs:694` 的 shape 错误（对外码 `jre_new_shape`）。实测诊断逐字匹配 694 模板、readers=`[23]`。

**既有先例（决定修复形状）**：`init.rs:676-678` 文档明写"The constructor's own copy is not a reader here: it is one of the three instructions this site owns, and `outside_readers` leaves the site's own instructions out"——即**站点自有的 `dup` 本就被排除在读者外**。本片要做的是把"被丢弃的 null-check 三元组"也归为**消费该实例的成员构造站点**自有，而非外层分配站点的游离读者。

## 关键未知（实现者必须先插桩确认，root 未定论）

`verify_member`（`init.rs:506` 调用）**先于**读者门（`init.rs:681`）执行。`verify_member` 有两条相关臂：
- **分配限定符臂**（929-976）：`nested_sites` 找到外层分配、`is_the_instance(physical_outer, …)`、`single_use_at(physical_outer, at)`——**但 javac 8 的 `dup(23)` 使外层实例有 2 个 use（`dup` + 成员构造器），`single_use_at` 要求恰 1 个 use 且位于 `at`，故该臂在真 javac 8 下应失败**（这是 root 的读码推断，**未插桩确认**）。
- **受检限定符臂**（977-998，`[qualifier,copy,check,pop]`）：getClass 片已让它认 `getClass` 拼写，但它要求 `qualifier` 是 `Operation::Load`（局部/参数），而分配限定符形的限定符是 `new`（非 Load），故**不匹配**。

**故第一道门可能是 `verify_member` 的分配限定符臂（`single_use_at` 失败 → `jre_new_member_order` 或 `jre_new_shape`），而非读者门 687**——getClass 片实现者的停手报告归因到前者（`jre_new_member_order`），root 的表层实测看到后者（687/694）。**两者不矛盾**：`verify_member` 失败会让成员证明为 `None`，随后读者门因 `dup(23)` 未被任何站点认领而独立拒绝。**实现者必须以插桩确认：真 javac 8 的 `N1.main` 究竟先撞哪道门、两道门是否都要改。** 这是本片 task 1.1 的首要义务，不得跳过。

## 决策 1（待插桩确认后二选一，或两者都做）

**方向 A（若第一道门是读者门 687）**：把"被丢弃的 null-check 三元组"纳入消费该实例的站点的自有指令集——即当外层实例被一个成员构造站点用作分配限定符、且其后紧跟 `dup; <discarded null check>; pop`（复用 `facts.rs::is_discarded_null_check`）时，该三元组归成员构造站点所有（加入其 `owned`/`nested_produced_by`），从而 `outside_readers` 不再把 `dup(23)` 计为外层分配的游离读者。**健全性**：只有"被丢弃"（`pop` 紧随、返回值不被消费）的 null-check 才归站点；用户显式 `o.getClass();`（其值被后续语句读、或非紧随 `pop`）不归入，保持 getClass 片负例 C 的纪律。

**方向 B（若第一道门是 `verify_member` 分配限定符臂的 `single_use_at`）**：放宽该臂，使其接受"外层实例有一个额外的、被丢弃的 null-check 读者"——即 `single_use_at` 改为"除被丢弃的 null-check `dup` 外，恰有一个 use 位于 `at`"。**但这会放宽 `single_use_at` 的语义**，须极其谨慎：`single_use_at` 是防止外层实例被多处消费（那会使 `new Outer()` 无法拼成单一表达式）的关键不变量。放宽须严格限定为"额外的 use 恰是被 `is_discarded_null_check` 证明的 null-check 的 `dup`，且该 `dup` 的值只被紧随的 check+pop 消费"。

**root 倾向方向 A**（把三元组归站点自有，与既有"构造器自有 dup 不是读者"的先例同构，不触碰 `single_use_at` 这一核心不变量），但**最终以插桩结果为准**——若第一道门确是 `verify_member`，则方向 A 单独不够（成员证明先失败，根本到不了读者门），须先做方向 B 或调整 `verify_member` 的调用时序。**实现者不得预设方向，须按 1.1 插桩结论选择，并在报告中说明为何该方向是第一道门的正解。**

## 决策 2：复用已交付谓词，不新增拼写判据

识别 null-check 一律调用 `facts.rs::is_discarded_null_check(kind, owner, name, descriptor, interface_reference)`（getClass 片交付，`NullCheckSpelling{RequireNonNull,GetClass}` 成对匹配、叉积落空）。**不得**在本片再写一遍 `getClass`/`requireNonNull` 的字面比较——那会产生第二套可漂移的拼写判据，正是 getClass 片决策 1 要避免的。

## 决策 3：不引入版本判据

同 getClass 片决策 2：两种产物 major version 均 52，`java_release`/`major_version` 无法区分，且违反 `classfile.rs:139-141`"never over the compiler that produced it"。修复认的是"被丢弃的 null-check"这一**事实**，不是"javac 8"这一**来源**。

## 验证标准（可证伪）

1. **主锚（真 javac 8）**：冻结 `N1.java`（DT-03 巡查锚）以 Corretto 1.8.0_432 编译，`N1.main` 的 `new N1().new Inner(3).total()` 折叠为源级限定语法，`N1` 族源码区 quotes 从 13 → 0（或与 javac 23 腿同形）；渲染源集 `javac --release 8` exit 0，`java -Xverify:all` 输出 `10`/`7`/`13` 与原 class 逐行一致。
2. **javac 9+ 零回退（逐字节）**：同一 `N1.java` 的 javac 23 腿渲染逐字节不变（本就无检查、已恢复）；getClass 片的 8 个 `requireNonNull` fixture + 参数限定符/隐式 this 形（`N1x`/`Wrap`）逐字节不变。
3. **参数限定符形零回退**：getClass 片已修的 `outer.new Inner(9)`（`N1x`/`Wrap`）不受本片影响（本片只动分配限定符路径）。
4. **健全性负例**：(a) 用户显式 `o.getClass();` 语句 + `o.new In()`（getClass 片负例 C）仍不误折叠；(b) 外层实例被**多处**真实消费（非被丢弃的 null-check）仍拒（`single_use_at`/读者门的核心不变量不破）；(c) null-check 的返回值**未被丢弃**（被后续读）则不归站点、仍拒。
5. **corpus 双腿扫描**：差异类只应是此前因分配限定符 `getClass` 形被拒的类；出现任何 `requireNonNull` 形或参数限定符形差异即回退失败，停下报告。
6. **门禁**：`cargo test --workspace --tests --locked --no-fail-fast`（基线 **301 目标 / 2970 passed / 0 failed**）、fmt、CI-exact clippy、`openspec validate --all --strict`（**274 项**）、`git diff --check`、新增 fixture 后再生 corpus fingerprint。

## Open Questions

1. **第一道门究竟是读者门 687 还是 `verify_member` 分配限定符臂？**（决策 1 的方向取决于此，task 1.1 插桩回答。）
2. **`verify_member` 与读者门的调用时序**（506 vs 681）是否需要调整，还是两门各自独立加固即可？（若成员证明先失败，方向 A 单独不生效。）
3. **分配限定符 + 成员构造器带实参**（`new Outer().new Inner(arg)`，实参非空）是否与无实参形（`new Outer().new Inner()`）走同一路径？root 的 `N1.main` 锚是 `new Inner(3)`（带 int 实参），故本片锚已覆盖带实参形；但纯无实参形须另冻一个对照，确认两者都被修（handoff "验收锚不得是唯一正例"）。
