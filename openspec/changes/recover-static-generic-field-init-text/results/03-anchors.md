# 任务 3.1/3.2 证据：主锚、零回退、corpus 差分分类（coder，2026-10-06）

修复落点：`src/facade.rs` 的 `project_static_fold_owner_texts` 字段重拼循环（机制与门控实验见
[`01-mechanism.md`](01-mechanism.md)）。diff = 1 文件 13+/3-，只改该循环的落点计算与注释。

## 3.1 主锚：两条腿、零损坏文本、编译与行为

harness：`/tmp/rsgfi/harness.sh`（渲染 → 自述头断言 → `Holdava` 计数 → 渲染文本按 public 类名写成
`<CLASS>.java` → JDK23 `javac --release 8 -Xlint:-options` 与 Corretto 8 `javac` 两腿编译 →
`-Xverify:all` 运行 → 与原件实测输出比较）。fixture = `tests/fixtures/recover-static-generic-field-init-text/`
的冻结字节（v8 腿与巡查 fixture `cmp` 相同，SHA 见 fixture README）。

```text
== baseline HEAD=dccd21c3（父提交二进制）==
class=MN label=base broken=2 rel8_exit=1 javac8_exit=1 original='ab5' rel8_run='-' javac8_run='-'
class=RG label=base broken=2 rel8_exit=1 javac8_exit=1 original='y7insf5\n2' rel8_run='-' javac8_run='-'
class=SG label=base broken=2 rel8_exit=1 javac8_exit=1 original='ab5' rel8_run='-' javac8_run='-'
class=P1 label=base broken=0 rel8_exit=1 javac8_exit=1 original='1' rel8_run='-' javac8_run='-'
（两条腿逐字节相同的渲染与相同的退出码）

== patched（本 change 二进制）==
class=MN label=patched broken=0 rel8_exit=1 javac8_exit=1 original='ab5' rel8_run='-' javac8_run='-'
class=RG label=patched broken=0 rel8_exit=1 javac8_exit=1 original='y7insf5\n2' rel8_run='-' javac8_run='-'
class=SG label=patched broken=0 rel8_exit=0 javac8_exit=0 original='ab5' rel8_run='ab5' javac8_run='ab5'
class=P1 label=patched broken=0 rel8_exit=0 javac8_exit=0 original='1' rel8_run='1' javac8_run='1'
（两条腿逐字节相同的渲染与相同的退出码）
```

`broken` = `grep -c 'Holdava'`（验收指定判据）⇒ `MN`/`MN8` 两腿 2 → 0 ✓。渲染文本 diff（父提交 →
修复后）**只**有字段声明行：

```diff
--- v8/MN ---
5c5
<     static Hold f1 = new MN$Holdava.lang.Object) "a");
---
>     static Hold f1 = new Hold((java.lang.Object) "a");
8c8
<     static Hold f2 = new MN$Holdava.lang.Object) "b");
---
>     static Hold f2 = new Hold((java.lang.Object) "b");
--- v8/RG ---
5c5
<     static Hold field = new RG$Holdava.lang.Object) "sf");
---
>     static Hold field = new Hold((java.lang.Object) "sf");
11c11
<     static Hold nested = new RG$Holdava.lang.Object) new RG$HolHold.lang.Object) java.lang.Integer.valueOf(5)));
---
>     static Hold nested = new Hold((java.lang.Object) new Hold((java.lang.Object) java.lang.Integer.valueOf(5)));
--- v8/SG ---
5c5
<     static Hold f1 = new SG$Holdava.lang.Object) "a");
---
>     static Hold f1 = new Hold((java.lang.Object) "a");
8c8
<     static Hold f2 = new SG$Holdava.lang.Object) "b");
---
>     static Hold f2 = new Hold((java.lang.Object) "b");
--- v8/P1 ---
4c4
<     static Box b = new P1$Box;
---
>     static Box b = new Box(1);
（v8-javac8 腿逐行同形；`cmp` 实测两腿渲染逐字节相同）
```

### 3.1.1 边界（如实记录）：`MN`/`RG` 的整类 `javac` 仍非 exit 0

验收句写的是"整类 `javac --release 8` exit 0"。实测：**该句在 `MN`/`RG` 上不可达**，且原因与本 change
的文本无关——父提交的解析错误被修好后，`MN` 渲染单元只剩**一个**错误，落在嵌套成员类 `Hold<T>` 自己的
擦除配对上：

```text
MN.java:41: 错误: 不兼容的类型: Object无法转换为T
            this.v = arg1;
                     ^
  其中, T是类型变量:
    T扩展已在类 Hold中声明的Object
1 个错误
```

即：`Hold<T>` 的字段 `Signature`（`TT;`）被投影为 `T v;`，而它的构造器 `Signature`
（`<T:Ljava/lang/Object;>(TT;)V`）被拒（`ordinary_generic_source_unproved`，注释逐字保留），
构造器因此写成 `Hold(java.lang.Object arg1)` —— `this.v = arg1` 是把 `Object` 塞给 `T`。该配对在
`--policy single-class` 渲染里**同样存在**（同一段成员文本），属于投影域的既有未决项，本 change 的
Non-Goal（"本片不修投影本身，只修文本"）明确排除。

巡查的 `results/javac-errors.txt` 只记了 2 条解析错误，是因为 javac 在解析阶段失败后**不再进入属性
检查阶段**（`-XDshould-stop.ifError` 默认行为），所以巡查证据无法说明修好解析错误后是否还有别的错误。
这条差异请 root 在验收时裁定（见 `01-mechanism.md` 末节）。

**在本 change 作用域内**，"可编译 + 行为一致"由两个对照锚完整交付（两条腿、两个编译器）：

| 锚 | 形状 | 修复后 `javac --release 8` | 修复后真 javac 8 | 运行输出 | 原件输出 |
| --- | --- | --- | --- | --- | --- |
| `SG` | 与 `MN` 同形（静态泛型字段 + `field_generic_body_unproved` 拒绝 + 内联初始化），嵌套类成员无 `Signature` | exit 0 | exit 0 | `ab5` | `ab5` |
| `P1` | 同机制的非泛型对照 | exit 0 | exit 0 | `1` | `1` |

`MN`/`RG` 的行为一致性用"仅手工修正那一处越界配对"的探针测得（`Hold(java.lang.Object arg1)` →
`Hold(T arg1)`，一处字符串替换，其余逐字节为渲染原文）：

```text
v8/MN(probe) rel8 OK run=ab5      v8/MN(probe) javac8 OK run=ab5
v8/RG(probe) rel8 OK run=y7insf5\n2   v8/RG(probe) javac8 OK run=y7insf5\n2
v8-javac8/MN(probe) rel8 OK run=ab5   v8-javac8/MN(probe) javac8 OK run=ab5
v8-javac8/RG(probe) rel8 OK run=y7insf5\n2   v8-javac8/RG(probe) javac8 OK run=y7insf5\n2
```

即：修复后的 `MN` 渲染文本（除该越界配对外的每一字节）可编译并打印与原件相同的 `ab5`。
（任务书写的 `a b 5` 与实测不符：`MN.main` 打印 `f1.v + f2.v + f3.v` 的字符串拼接，实测为 `ab5`，
两条 fixture 腿一致；本 change 以"与原件逐字相同"为准。）

## 3.2 零回退

### 3.2.1 `MN.f3`（实例字段退化路径）逐字节不变

`MN` 渲染 diff（上引）不含 `f3` 的任何行：`    Hold f3;` 与构造器里的
`        this.f3 = new Hold((java.lang.Object) java.lang.Integer.valueOf(5));` 在父提交与修复后逐字节相同，
两条腿相同。

### 3.2.2 `RG` 宿主的非静态字段与方法不受影响

`RG` 渲染 diff 只有 `field` 与 `nested` 两行（都是静态字段声明）。`RG.names`（`static java.util.List`，
其初始化不点名被折叠类，声明里池形名只出现一次）**不在 diff 中**；`pick`/`consume`/`newInner`/`main`
的方法文本逐字节不变（`Hold local3 = new RG().newInner();`、`return new Hold((java.lang.Object) "in");`
等行两侧相同）。

### 3.2.3 corpus 双腿扫描

脚本：[`02-corpus-sweep.sh`](02-corpus-sweep.sh)（三档：A 单类姿态全语料、B 折叠姿态每个 loose 根类、
C 每个已提交 jar 的每个 `.class` 条目；两二进制 = 父提交 `dccd21c3` 构建 vs 修复后构建）。
自检先行：`MN` 家族 `Holdava` 2 → 0（已知正例）、`MN` 单类姿态逐字节相同（已知负例）、
无关 jar `bs.jar!BS.class` 逐字节相同（已知负例）、**pass A 全语料 moved 必须为 0**（否则脚本退出非零）。

结果（全文见 [`02-corpus-sweep.out`](02-corpus-sweep.out)）：

```text
SELF-TEST OK: stated_name reads MN$Hold from the nested fixture and nothing from a non-class
SELF-TEST OK: MN family Holdava 2 -> 0; MN single-class byte-identical; bs.jar!BS.class byte-identical
pass A/B loose candidate classes: 2752
pass A: moved=0 unrendered=1
pass B: root classes=2140 moved=10 unrendered=2
archive candidate classes: 738
pass C: moved=0 unrendered=1
moved classes: single-class=0 fold=10 jar=0 total=10
unrendered candidates: A=1 B=2 C=1
```

10 个差异类 = 巡查的 `MN`/`RG` 两个锚 + 本片冻结的四类 × 两条腿；18 行变更**全部**是字段声明行；
每类池形名行数 2 → 0。逐类分类、未渲染候选的逐条解释，以及**脚本首版 `stated_name` 假零**（读到
`javap` 的 InnerClasses 注释行 → 家族 jar 缺成员 → 折叠被拒 → 两二进制同输出，pass B 假 0）的更正记录，
见 [`02-corpus-delta.md`](02-corpus-delta.md)。
