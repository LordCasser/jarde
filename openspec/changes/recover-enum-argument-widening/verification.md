# 验证（change `recover-enum-argument-widening`）

与 `recover-charsequence-argument-widening`、`recover-comparable-argument-widening` **一次实现**
（同一表族、同一判定函数、同一测试入口）。

## 落点复核（与 proposal 预审计的差异，实测为准）

预审计记“平台扩宽表新增 `java.lang.Enum` 行”。**实测**：`EnumSet.of` 的 `java.lang.Enum` 位在 HEAD
**已由 `recover-platform-interface-argument-widening` 落地的快照单边证明覆盖**——证据是同一 fixture 的
判别实验：

| 条件 | BCI 6（`java.lang.Enum` 位） |
| --- | --- |
| 容器含 `OB$Flag.class`（快照内枚举类，header 逐字 `java/lang/Enum`） | 该拒绝**消失**（改前记录有、HEAD 无） |
| `--policy single-class`（快照内无枚举类） | 该拒绝**逐字回归** |

故 `java.lang.Enum` **不能**也不需要一个封闭表行：呈现类型是**用户枚举类**（javadoc 无法封闭枚举其名），
而该类自己的 class-file header 就是完整证明。**本片实测出的真实剩余拒绝是锚同方法的级联伴行**
`retainAll(fs)`：`java.util.EnumSet presents java.util.Collection`（BCI 12）——枚举族的 `java.util.EnumSet`
自身的 javadoc 行，正是 proposal 追记里“实现时一并覆盖（同一 widen 通道）”的那一行。

## 实现（`crates/jarde-java/src/build.rs`）

`platform_interface_argument_widens` 的第四张 `const` 行表（枚举族集合类型；落点与三张姊妹表同一函数）：

| 行 | 呈现类型 → 目标 | release 8 事实 |
| --- | --- | --- |
| 1 | `java.util.EnumSet` → `java.util.AbstractSet` | header `extends java.util.AbstractSet<E>` |
| 2–4 | `java.util.EnumSet` → `java.util.Set` / `java.util.Collection` / `java.lang.Iterable` | implemented-interface 列表（`AbstractSet → AbstractCollection → Collection → Iterable`），读法与 java.util 表的 `Properties → java.util.Map` 同规 |

`EnumSet` 的 `Cloneable`/`Serializable` 关系不入表（本片钉的是集合位；Serializable 表是 java.lang 九行）。
既有 java.util 表**逐字未动**；由本表命中 `EnumSet → Collection` 与该表自身的答案（无 EnumSet 行）构成
通道并集，与既有“多证明通道各自部分覆盖、OR 作答”的结构一致——代码注释已写明这一分工。

## 锚实测 vs 预期（两条 javac 腿逐字一致；`essential` + source map 入口）

| 锚 | 预期 | 实测 |
| --- | --- | --- |
| `OB.flags`（巡查主锚） | 整方法恢复、行为一致 | `EnumSet.of((java.lang.Enum) OB$Flag.A, (java.lang.Enum) OB$Flag.C)` + `local1.retainAll((java.util.Collection) arg0)` + `contains` 三行全出；该方法 refusals=0 |
| `EN.flags`/`EN.three`（冻结形） | 同上 + 变长形 | 文本逐字钉（双腿一致） |
| `EN.same` | 真 `java.lang.Enum` 声明型实参**不引入 cast** | `return java.util.EnumSet.of(arg0, arg1);`（同型回答） |
| `EN.eq`/`h`/`str` | 无枚举路径零漂移 | 文本逐字等于 objects-enumset 巡查记录的健康面 |
| `EN.main` | 全锚调用 | 文本逐字钉 |
| `OB.main` | 拒绝集不因本片漂移 | 与巡查记录**逐字节一致**（7 行，全为 BCI 79/85/87 的嵌套数组/copy 家族，无 widening 行） |
| 负例 `ENX.platform` | 平台枚举仍拒 | `java.lang.Thread$State`（不在快照、不入表）拒绝文本逐字，计数恰为 1 |
| spec 负例（“非枚举实参”） | 仍拒 | `Objects.*` 三位点零漂移（上表）+ 单元级表拒绝（`String → java.lang.Enum` 等，见 build.rs 单测） |

## 冻结与行为

- `tests/fixtures/recover-enum-argument-widening/`（`EN`/`ENN`/`ENT`/`ENX` 源 + 两腿 class + README 记
  sha256、腿命令与复现的既有边界）；
- `tests/recover_platform_implementer_argument_widening.rs`：`the_enum_positions_are_presented`（7 文本
  逐字 + 零引注）、`the_patrol_nested_enum_shape_presents_its_call`（巡查 `$` 嵌套形）、
  `the_platform_enum_still_refuses`（1 负例 + 计数 1）——两腿各断言一次；
- ignored replay：`EN` 剥离后双腿编译运行，答案 `hasA/no/true/true/0/dflt` 与 fixture 自身 class
  逐字一致（实测通过）。`ENT.java` 用 fixture 自身声明（枚举类自身的渲染不是合法源码单元：它声明
  `super(name, ordinal)`，源码不可写）；`EN` 的方法文本是**渲染文本**。
- `ENN`（巡查的 `$` 嵌套形）只作文本锚不进 replay：两腿的**类文本 fold** 对嵌套枚举投影不同
  （javac 23 腿发布 `enum Flag` 声明并重拼引用；真 javac 8 腿保持池形、无声明）——既有 fold 差异，
  与本 change 的实参扩宽无关，如实记录在 fixture README。

## 零回退与边界

- `OB` 的 `Objects.*` 健康面、`EnumSet.noneOf(Flag.class)`（Class 形参）逐字未动；
- 既有 java.util 表、Throwable 通道、数组闭集、Object 回答、snapshot 层逐字未动；
- 平台枚举（`Thread$State` 等）仍拒——封闭表不能凭记忆枚举 JDK 枚举，快照/classpath 证明是另一条地面。

## 移动的既有 pin（如实记录，1 处）

`tests/p3_platform_collection_widening.rs` 的 CWN table-out 计数 7 → 6：那条正是
`java.util.EnumSet presents java.util.Set`——本片落的 `EnumSet` 自身 javadoc 行（`Set` 列）把它从
“本 slice 枚举域之外”移到“有行作答”。该行在 objects-enumset 巡查追加里本就登记为 **OB 锚的级联
伴随**（probe 亦证不是独立失败面），故这是本片范围的预期移动；断言改为“该拒绝句已不在文本中”，其余
六条 table-out 拒绝逐字保留（`CWN$MyList`/`CWN$MySubList`/`ConcurrentHashMap`/`IdentityHashMap`/
`Date`/`CWN$Kind`）。**注意**：`CWN$Kind → java.lang.Enum` 仍拒的条件之一是**该枚举类不在本测试的
快照里**（测试只开 `CWN.class`）——与巡查判别实验同因（枚举位靠快照单边证明，本表不答用户枚举）。

## 门禁

与三片共用（见 `recover-charsequence-argument-widening/verification.md` 的门禁表）：
`cargo test --workspace --tests --locked --no-fail-fast` exit 0（308 targets / 3021 passed / 0 failed /
52 ignored）、fmt 干净、CI-exact clippy exit 0、`openspec validate --all --strict` 300/0、
`git diff --check` 干净；语料指纹再生、fixture 人口 `(687, 2932, 282, 1827, 8)`（+22 类/94 body，
含本片 5 类 × 2 腿）。
