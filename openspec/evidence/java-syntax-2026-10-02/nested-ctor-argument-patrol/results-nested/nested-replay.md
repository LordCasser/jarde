# 嵌套构造实参走查扩展重放（2026-10-02）

基线 = 主线 `05a8bf2a` 的恢复代码（`variants-nested/X3.before.jarde.java`、`X4.before.jarde.java` 与冻结 fixture 输出 `../results/X1.jarde.java`、`../results/X2.jarde.java`）；实施 = `recover-nested-ctor-argument-sites` 走查扩展。所有恢复文本来自 `jarde-cli class-source --policy single-class --format text`；fixture SHA 复核：`X1.class 8bb00653…`、`X2.class 84cdb5c5…` 与 [../results/fixture-sha256.txt](../results/fixture-sha256.txt) 一致，变体 SHA 见 [../variants-nested/sha256-variants-nested.txt](../variants-nested/sha256-variants-nested.txt)。

## 走查定位（1.1）

- 落点：`crates/jarde-java/src/init.rs` `verify` 的构造调用选择——原 `block.iter().skip(index + 2).find(owner == ty && <init>)` 在 X2.nested 的实参区间先遇同类内层 `invokespecial@12`，接收者检查（"the constructor at BCI 12 is called on a value this allocation did not produce"）整组拒绝；X1.main `@43/@46` 同因。
- 扩展：外层实参扫描遇 `new`+`dup` 起始且非 `reserved`/`chains.owns` 的内嵌序列时按同一 `verify` 递归证明（深度上限 2，`MAX_NESTED_CONSTRUCTION_LAYERS`），成功则跳过其闭区间续扫，外层构造调用选到第一个真正属于自己的 `<init>`；递归失败不跳过，外层按原检查拒绝（负例消息与主线一致）。内嵌站点由既有主体走查独立登记并由 `render_value → site_of → new_expr` 复用构造拼写呈现，无新呈现函数。
- 新增拒绝链：值不是外层任一实参（`jre_new_shape`，"completes inside the construction at BCI …"）与嵌套区间跨异常边界（`jre_new_nested_exception_boundary`，与 concat/inline char[] 同一判据）。

## 变体前后（1.2；`X3.before`/`X4.before` vs `X3-nested`/`X4-nested`）

| 场景 | 主线 | 实施 |
| --- | --- | --- |
| X3.doubleNested `new X3$TwoNested(new X3$B("y"), new X3$C("z"))`（双内嵌实参，不同类） | 整方法退化（`@bytecode 0/3` + `28 25 22`） | `return java.lang.String.valueOf((java.lang.Object) new X3$TwoNested(new X3$B("y"), new X3$C("z")));` |
| X3.secondPosition `new X3$Tagged("first", new X3$B("second"))`（内嵌于第二参位） | 整方法退化 | 完整恢复 |
| X3.sameClassTwice `new X3$TwoSame(new X3$B("1"), new X3$B("2"))`（同类内嵌两次） | 整方法退化 | 完整恢复 |
| X4.threeLayer `new X4$Top(new X4$Mid(new X4$Leaf("z")))`（三层） | Top、Mid 拒绝（`@bytecode 0/3/4/7`），Leaf 站点已证 | **Top 保持拒绝并登记**（"the construction at BCI 8 completes inside the construction at BCI 0 …"）；Mid+Leaf 站点翻为已证（`@bytecode` 只剩 `0/3` + 语句引述 `26 23 20`），方法仍退化（三层上限 2） |
| X4.doubleUse `new X4$Tag("t", kept = new X4$Val("assigned"))`（内嵌双用途） | Tag、Val 拒绝 | **消息与主线逐字一致**（Tag：`an Allocate … between the allocation's copy and its constructor call`；Val：`read only by instructions this build quotes (BCIs 15)`），整方法仍退化 |
| X4.crossBlock `flag ? new X4$Val("a") : new X4$Val("b")`（跨块，附加边界） | Tag "never constructed" | 同消息保持拒绝 |

既有形态逐字不变：X2.single / X2.plainNested / X2.nestedNew / X1.wrapCtor（throw 位）/ X1.wrapInitCause / X1.readCause 与冻结输出 diff 为空（仅目标行翻转，测试 `nested_ctor_argument_sites.rs` 逐字钉死）。

## 三方对照（3.2；`three-way/`，固定 JADX dev）

原 class / 固定 JADX / Jarde class-source 三条腿各自 `javac --release 8` 重编后 `java -Xverify:all` 运行，stdout SHA 逐腿一致，stderr 全净：

| 类 | stdout SHA（三腿一致） | 内容 |
| --- | --- | --- |
| X1 | `e1c220f70795bf23a26384fbc43ccdcec4d5591fbba8b9b4308f57570e466335` | `empty` / `root` / `inner` |
| X2 | `f7d218ab6f4c295a142eb57188f882758712909187e31581d2fe9e4b697c1492` | `solo` / `outer` / `sb` |
| X3 | `7146cf772f30b167492c6f40852cc34859aea931cd542af7c95ec656dba26e1d` | `TwoNested(B(y),C(z))` / `Tagged(first,B(second))` / `TwoSame(B(1),B(2))` |
| X4（负例） | `26fa39f73c80744422176bfbc5441d654b6acc7b416d24a7dafbf5d9af205aee`（原/JADX 两腿） | `Top(Mid(Leaf(z)))` / `Tag(t,Val(assigned))/Val(assigned)` / `Tag(t,Val(a))` |

X4 的 Jarde 腿文本含引述不可编译（三层与双用途负例按设计保持拒绝，与 CF-17 `twrNamed` 同口径）。X3 的 Jarde 腿与其五个嵌套类的单独呈现一起编译（`X3$B` 等以顶层 `$` 名呈现，二进制名一致）。

## 门禁（3.1）

- `cargo test --workspace --tests --locked --no-fail-fast`：**2803 通过、0 失败**（277 个测试二进制全绿；一次并行运行中 `protected_gateway_with_exception_edge_is_refused`/`p4_plugins` 单发失败，clean 复跑与全量复跑未复现，按任务书 flake 口径判定）。
- `cargo fmt --all -- --check`：通过。
- CI 同款 clippy（`ci.yml` 全量 `-A` 清单——29 项，按 workflow 原文逐项取全——加 `-D warnings`）：零告警。
- `openspec validate --all --strict`：通过（240 项）。
