# 平台集合层级实参上转型实现证据（`recover-platform-collection-widening`）— 2026-10-02

[巡查 README](../README.md) 的实现侧证据：`crates/jarde-java/src/build.rs` 的
`platform_reference_argument_widens` 由单一 `List → Iterable` 对扩展为 JDK 8 `java.util` 集合
层级的闭集直接边 + 传递闭包 walk（先例 `java_lang_throwable_widens` 同款）。呈现走既有
`cast_argument`，保留要求类型拼写。回归 `tests/p3_platform_collection_widening.rs` 与 `build.rs`
单元测试 `platform_reference_argument_widening_reaches_exactly_the_java_util_table_ancestors`。

## 闭集表（40 行直接边）

每行 = JDK 8 `java.util` 集合类型 javadoc 声明的**一条**直接 `extends`（父类/父接口）或
`implements`（接口）。表在 [`results/collection-table-rows.txt`](results/collection-table-rows.txt)
（从 `build.rs` 的 `DIRECT_EDGES` 机械提取）。分组与 javadoc 来源：

| 组 | 行 |
| --- | --- |
| `Abstract*` 骨架 | `AbstractCollection implements Collection`；`AbstractList extends AbstractCollection implements List`；`AbstractSequentialList extends AbstractList`；`AbstractSet extends AbstractCollection implements Set`；`AbstractMap implements Map` |
| `List` 实现 | `ArrayList extends AbstractList implements List`；`LinkedList extends AbstractSequentialList implements List, Deque`；`Vector extends AbstractList implements List`；`Stack extends Vector` |
| `Set` 实现 | `HashSet extends AbstractSet implements Set`；`LinkedHashSet extends HashSet implements Set`；`TreeSet extends AbstractSet implements NavigableSet` |
| `Map` 实现 | `HashMap extends AbstractMap implements Map`；`LinkedHashMap extends HashMap implements Map`；`Hashtable implements Map`；`Properties extends Hashtable`；`TreeMap extends AbstractMap implements NavigableMap` |
| `Deque` 实现 | `ArrayDeque extends AbstractCollection implements Deque` |
| 接口链 | `Collection extends Iterable`；`List/Queue/Set extends Collection`；`Deque extends Queue`；`SortedSet extends Set`；`NavigableSet extends SortedSet`；`SortedMap extends Map`；`NavigableMap extends SortedMap` |

**有意不收**：`java.util.Dictionary`（`Hashtable` 的另一 javadoc 父类，非集合类型）；`EnumSet`/
`IdentityHashMap`/`PriorityQueue`/`WeakHashMap`/`EnumMap`/`AbstractQueue`（域外枚举类型）；
`java.util.Collections`/`Arrays` 工厂；`java.util.concurrent` 等子包；`Date` 等非集合 java.util 类型；
全部用户类与用户子类（升级路径未启用）。`java_release != 8` 一律拒绝。

## 机械核对（逐对 JDK 反射）

[`CollectionTableCheck.java`](CollectionTableCheck.java) 以运行时反射对表逐对断言，产物
[`results/collection-table-check.txt`](results/collection-table-check.txt)：

- **JDK 8**（Corretto `1.8.0_432`，发布版层级的权威运行时）五项全过：
  - A 每行都是该 JDK 的**直接**父类/父接口边（class：`getSuperclass()` 或 ∈ `getInterfaces()`；interface：∈ `getInterfaces()`）；
  - B 每行留在 `java.util`（唯一例外 `Collection → java.lang.Iterable` 根边，且只有 `Collection` 可命名它）；
  - C 域内 812 对上，walk 的可达性与运行时的 `isAssignableFrom` 完全一致；
  - D 40 行**恰为**域内全部直接边，无多无少（父类在域外的行如 `Hashtable → Dictionary` 不是行）；
  - E 12 个域外类型从不进入 walk（348 对），且 `Hashtable → Dictionary` 虽被运行时允许仍保持拒绝。
- **JDK 23**（`23.0.1`）同一脚本在 A 处**预期失败**（`LinkedHashSet → SequencedSet` 等 Java 21+
  新增接口）：这正是 `java_release == 8` 门禁的证据——发布版表不能跨版本泛化。

## 基线重放（tasks 1.1）

`fixture/G1.class` SHA-256 与 [results/fixture-sha256.txt](../results/fixture-sha256.txt) 一致
（`fabbfd8e…`）。`javap -c` 核对：`35: aload_0; 36: invokestatic max:(Ljava/util/List;)Ljava/lang/Comparable;`
——拒绝点即 BCI 36 的参数 0。可重放命令（工作树根）：

```
target/debug/jarde-cli class-source \
  --input openspec/evidence/java-syntax-2026-10-02/collection-widening-patrol/fixture/G1.class \
  --class G1 --policy single-class --format text
```

实现前输出与提交的 [results/G1.jarde.java](../results/G1.jarde.java) **逐字节一致**
（[results/G1-widen.base.txt](../results/G1-widen.base.txt)）：BCI 36 参数 0 拒绝
（`ArrayList presents … requires java.util.List … no safe reference conversion evidence`）、结果局部
`local1` 声明级联拒绝（BCI 54–77 引用残留）。实现后（[results/G1-widen.recovered.txt](../results/G1-widen.recovered.txt)）
`use` 完整：

```
java.lang.String local1 = (java.lang.String) max((java.util.List) local0);
java.lang.String local2 = (java.lang.String) pick((java.lang.Object) "a", (java.lang.Object) "b");
return local1 + ":" + local2;
```

整类仅剩 `autoboxLoop` 的 BCI 44 一处 `@bytecode`（装箱循环别名残留，另一家族，不在本片）；
`max`/`pick`/`main` 呈现与基线逐字不变。

## 变体族（tasks 1.2 / 2.2）

[CWV.java](CWV.java) → 冻结 [original/CWV.class](original/CWV.class)（javac 23.0.1 `--release 8 -g:none`）。
实现前 33 处拒绝（[results/CWV-widen.base.txt](results/CWV-widen.base.txt)），实现后 **0 处拒绝、0 个
`@bytecode`**（[results/CWV-widen.recovered.txt](results/CWV-widen.recovered.txt)），呈现保留要求类型：

| 组 | 复现行 |
| --- | --- |
| `Map` 实现 | `HashMap`/`TreeMap`/`LinkedHashMap`/`Hashtable` → `Map`；`Properties` → `Map`（两跳 `Hashtable`） |
| `Set` 实现 | `HashSet`/`TreeSet`/`LinkedHashSet` → `Set`；`TreeSet` → `SortedSet`/`NavigableSet` |
| `List` 实现 | `ArrayList`/`LinkedList`/`Vector` → `List`；`Stack` → `List`（两跳 `Vector`） |
| `Deque` 实现 | `ArrayDeque` → `Deque`/`Collection`；`LinkedList` → `Queue` |
| 接口链 | `List`/`Set`/`Queue` → `Collection`；`Collection` → `Iterable`；`Deque` → `Queue`；`SortedSet` → `Set`；`NavigableSet` → `SortedSet`；`NavigableMap` → `SortedMap` |
| 双跳/多跳 | `ArrayList` → `Collection` → `Iterable`；`TreeSet` → `NavigableSet` → `SortedSet` → `Set` |
| 嵌套泛型 | `ArrayList<List<String>>` → `List<List<String>>`（`nestedListSize`）与元素位 `elementList.get(0)` → `List<String>` |

负例族 [CWN.java](CWN.java) → 冻结 [original/CWN.class](original/CWN.class)：实现前 7 处拒绝，实现后
**7 处逐字保留**（[results/CWN-widen.recovered.txt](results/CWN-widen.recovered.txt) 与
[results/CWN-widen.base.txt](results/CWN-widen.base.txt) **逐字节相同**）：用户 `MyList extends AbstractList`
→ `List`、用户 `MySubList extends ArrayList` → `List`、`java.util.concurrent.ConcurrentHashMap` → `Map`、
`java.util.EnumSet` → `Set`、`java.util.IdentityHashMap` → `Map`、`java.util.Date` → `Comparable`、
用户枚举常量 `Kind` → `Enum`。原类 `java -Xverify:all` 运行输出一致
（[results/CWN-original-run.txt](results/CWN-original-run.txt)）。

## 三方对照（tasks 3.2）

固定 JADX dev CLI（`jadx-cli/build/install/jadx/bin/jadx`，1.5.6）。原 class / JADX Java 输入 /
Jarde 恢复各自 `javac --release 8` 重编 + `java -Xverify:all`，**逐路径一致**：

| fixture | 原 class | JADX | Jarde | 统一 SHA-256 |
| --- | --- | --- | --- | --- |
| G1 | `zeta:a`/`6` | `zeta:a`/`6` | `zeta:a`/`6` | `79f6081d…`（[G1-original-run.txt](results/G1-original-run.txt) / [G1-jarde-run.txt](results/G1-jarde-run.txt) / [jadx-G1-run.txt](results/jadx-G1-run.txt)） |
| CWV | 26 行标签 | 同 | 同 | `5e38f1ab…`（[CWV-original-run.txt](results/CWV-original-run.txt) / [CWV-jarde-run.txt](results/CWV-jarde-run.txt) / [jadx-CWV-run.txt](results/jadx-CWV-run.txt)） |

JADX 对 CWN 无独立运行列（其嵌套类非限定名不可独立编译）；CWN 的运行锚为原类。

## 测试与门禁

- `crates/jarde-java/src/build.rs`：`platform_reference_argument_widening_reaches_exactly_the_java_util_table_ancestors`
  （50 正例含跨层可达 + 24 负例：非 8 版本、域外 java.util 类型、子包、用户类、向下/兄弟方向、原始/数组形状/尖括号拼写）。
  既有 `List → Iterable` 断言并入同一测试并保持。
- `tests/p3_platform_collection_widening.rs`：G1 `use` 三语句与整类标记钉死、G1/CWV 重编运行对照、
  CWV 逐组呈现、CWN 七拒绝钉死、CWN 原类运行、预算 8 字节停止与取消原子性。
- 回归：`p3_throwable_wrap_arguments`（5/5）、`array_invocation_widening`（List→Iterable 与
  `no safe reference conversion evidence` 负例）、`p3_java_recovery` 全量套件内保持绿。
- `cargo test --workspace --tests --locked --no-fail-fast`：282 套 **2783 passed / 0 failed**；
  `cargo fmt --all -- --check` 干净；`.github/workflows/ci.yml` 完整 30 项 `-A` 清单 clippy
  `-D warnings` 通过；`openspec validate --all --strict` 237/237。