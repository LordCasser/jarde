# 枚举 String 实参投影的字段名硬编码（2026-10-04，root）——已验收能力只在自身 fixture 字段名上生效

按 Goal "持续巡查各个反编译语法分析模块，构造各种 Java 语法场景，编译，然后对比源码、jadx、jarde" 巡查**枚举常量专属体 + String 构造实参**形时发现的缺陷。主线 HEAD 二进制（`4e834142`，环 1 合入后重建）。

**结论：已验收切片 `recover-proved-string-arg-enum-constant-bodies`（8/8，提交 `7e49c75f`）的实现把字段名硬编码为 `op`，而 `op` 恰是其验收锚 `TestEnums2a/DoubleOperations` 的字段名——故该验收是自我实现的，声称的能力在真实输入上基本不生效。失败是响亮的（非法 Java → `javac` exit 1），非静默偏离。** 已立 spec [recover-enum-string-field-name-generalization](../../../changes/recover-enum-string-field-name-generalization/proposal.md)。

固定转录见 [fixture](fixture/)（六形源码 + 19 个 `.class` + `fam.jar`）与 [results](results/)（六份渲染全文 + SHA256）。

## 六形对照（仅字段名/getter 名不同，其余结构逐字节同构）

探针模板（`demo` 包，实现 `IOps { double apply(double,double); }`，两常量各带 String 实参与常量专属体）：

```java
package demo;
public enum C1 implements IOps {
    TIMES("*") { @Override public double apply(double x, double y){ return x*y; } },
    DIVIDE("/") { @Override public double apply(double x, double y){ return x/y; } };
    private final String op;
    C1(String op){ this.op=op; }
    public String getOp(){ return op; }
}
```

| 探针 | 字段名 | getter | 渲染结果 |
| --- | --- | --- | --- |
| C1 | `op` | `getOp` | **投影成功**：`TIMES("*") { public double apply(…) { return arg1 * arg3; } }, DIVIDE("/") { … };` + `private demo.C1(java.lang.String arg0) { this.op = arg0; }` |
| C3 | `op` | `getT` | **投影成功**（getter 名无关） |
| C2 | `t` | `getT` | 不投影：`public static final demo.C2 TIMES;` / `public static final demo.C2 DIVIDE;`（字段声明式常量） |
| C4 | `t` | `getOp` | 不投影 |
| C5 | `value` | `getValue` | 不投影 |
| C6 | `x` | `getX` | 不投影 |

**判别变量是字段名本身。** root 另测以下变量均**不改变**结论（即都不是判别变量）：常量名（`A/B` vs `TIMES/DIVIDE`）、包（默认包 vs `demo`）、常量体方法参数个数（0 参 `tag()` vs 2 参 `apply(double,double)`）、是否 `@Override`、方法声明位置（接口声明 vs 枚举自身 `abstract`）、jar 组成（是否含 `Runner.class`）、类 flags（两形均 `0x4421` = `ACC_PUBLIC|ACC_SUPER|ACC_ABSTRACT|ACC_ENUM`）。

## 严重度实测（响亮失败，非静默偏离）

不投影时枚举被呈现为"字段声明式常量"，这是**非法 Java**。root 实测（C1/C2 渲染产物 + **匹配接口** `IOps { double apply(double,double); }`，独立目录、文件名与 public 类名一致、`javac` 退出码直判而非经管道）：

```text
# C1（字段名 op，投影成功）——常量体 apply 方法保留
javac --release 8 -Xlint:-options -d o1 demo/IOps.java demo/C1.java
  exit 0

# C2（字段名 t，不投影）——常量退化为字段声明、常量体 apply 丢失（self-check: grep apply demo/C2.java 无命中）
javac --release 8 -Xlint:-options -d o2 demo/IOps.java demo/C2.java
  exit 1
  错误: 此处需要枚举常量
  错误: 需要')'或','
  错误: 需要';'
```

即同一模板下，**仅字段名不同**使完整源集从 `javac` exit 0 翻转为 exit 1。渲染文本本身带 `// jarde: not a compilable project` 头部声明与 `class_generic_source_unproved` / `jvm_signature_erasure_mismatch` 诊断，且不含 `@bytecode` 引注（因为整类回落物理文本，而非逐指令引注）。**故不触发"可编译但行为不同"这一核心不变量**，但属真实能力缺口：现实代码几乎不会把枚举字段命名为 `op`。

## 代码级根因（root 读码定位，行号随主线漂移、以锚点名为准）

五处 `op` 字面量：

| 落点 | 内容 |
| --- | --- |
| `src/facade.rs:14197` | 构造器 BCI 6 的 `putfield` 必须 `name == b"op"`，否则 `Err("the String constructor does not preserve Enum and op semantics")` |
| `src/facade.rs:16009` | 查找源字段时要求 `field.item.name.raw().0 == b"op"` |
| `src/facade.rs:16023` | 唯一性检查按 `== b"op"` 计数 |
| `src/facade.rs:16026` | 再核对 `op_field.item.name.raw().0 != b"op"` |
| `src/facade.rs:16031` | **发射文本字面量** `"    private {}(java.lang.String arg0) {{\n        this.op = arg0;\n    }}\n"` |

**数据通路缺口**：14197 处 `EnumCodeReference::Field { owner, name, descriptor }` 的 `name` **就是已证字段名**（它是构造器字节码里真实 `putfield` 的目标），但被 `matches!` 消费后即丢弃——承载证明结果的 `PendingEnumConstructorEdge`（`facade.rs:13942`）只有 `caller` / `call_bci` / `target_owner` / `target_descriptor` 四项，**不携带字段名**。故修复须打通"已证名 → 发射处"的通路，并把 16031 的 `this.op` 改为 `this.<已证名>`。

**spec 无字段名条件**：root 实测 `grep -niE "op\b|字段名|field name"` 于该 change 的 `specs/java8-recovery/spec.md` **无命中**；其 Requirement 原文的能力边界是"每个常量只带一个可无损拼写的 ASCII 字符串字面量源实参"。故实现**窄于其自身 spec**。

## root 取证过程中的自查纠错（按 handoff "验证脚手架必须先自检" 纪律登记）

1. **误判"C1 也不可编译"**：首轮比对时 root 用的 `IOps` 声明的是 `String tag()`，而 C1/C2 的常量体实现的是 `double apply(double,double)`，接口与常量体不匹配 → javac 报"不是抽象的, 并且未覆盖抽象方法"。**该失败是 root 脚手架造出的，与被测命题无关**；C1（`op` 名）实际是已验收的 DT-12 同形，投影正确。
2. **普查/隔离实验的三次静默失败**：(a) `sed` 的 `1i` 语法在 macOS BSD sed 下报错致 `pk` 腿未生成；(b) `jar cf ../pk.jar …` 在子目录内执行丢失包前缀，jar 条目为 0，渲染全空却被读成"不投影"；(c) python 模板 `{{` 转义错误致 javac 失败、无 `.class` 产出。三次都因 root 加了"自检 jar 条目数 / 打印期望值"才被发现。
3. **相对路径 `cp` 静默失败**：冻结 fixture 时先 `cd /tmp/...` 再用相对 `$E` 路径 `cp`，文件落到错误位置；改用绝对路径并以 `ls` 复核后才确认真正冻结。

**教训（已固化进 handoff）**：脚本产出为零/为空/全同值时默认怀疑脚本而非被测物；`grep` 计数（如 `const-list=0`）不等于"未投影"，必须读实际渲染文本才能下结论——本会话 root 三次因只看计数而险些得出错误判别变量。

## 处置

- **已修复并 root 验收（2026-10-04 同日）**：[recover-enum-string-field-name-generalization](../../../changes/recover-enum-string-field-name-generalization/proposal.md)（合并 `4734a4a1`；含五处落点、数据通路缺口、Non-Goals，以及"`TestEnums2a` 呈现必须逐字节不变"这一零回退锚——它同时是"名字恰为 `op`"与"名字由字节码证明"两种实现的共同正例）。**验收结果**：root 用自己的六形探针复测，判别变量已消除（`t`/`value`/`x` 三形由不投影转为投影，`op` 两形零回退）；**跨时零回退**——以 `javac -g:none` 重编 DT-12 冻结源后用合并后二进制渲染 `demo.DoubleOperations`，与本目录取证时对照的 **2026-09-27 已验收转录逐行 diff 完全一致**；新冻结非 `op` 正例（字段名 `label`）渲染源集 `javac` **exit 1→0**、`java -Xverify:all` 输出与原 class 逐行一致（含匿名子类名与 ordinal）。DT-12 账本状态已由"有差距"恢复为"冻结差距已修复"。
- **优先级**：属**已验收能力的正确性缺口**（非新能力），按 root "correctness-before-capability" 纪律排在纯呈现润色之前——本会话据此把它插到 5.3 链环 2 之前派发。其失败是响亮的（不产生静默偏离），故非紧急正确性事故，但真实能力缺口须修。
- **附带登记（已由修复片落实）**：`recover-proved-string-arg-enum-constant-bodies` 的验收应补一个**非 `op` 字段名**的冻结正例，否则同类"验收锚恰为满足硬编码而设计"的缺陷无法被 CI 发现——修复片已冻结 `tests/fixtures/enum-string-field-name/`（字段名 `label`）并加 CI 测试 `tests/enum_string_field_name.rs`（含 `assert!(!text.contains("this.op"))` 守卫），该缺陷类现已被 CI 自动捕获。此教训已固化进 handoff "验收锚不得是唯一正例"纪律。

原 class 为行为基准（各探针原 class 均可编译运行，`TIMES`/`DIVIDE` 的 `apply` 语义为乘/除）。
