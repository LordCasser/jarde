# 任务 3.2 证据：corpus 三档差分与逐类分类（coder，2026-10-06）

脚本 [`02-corpus-sweep.sh`](02-corpus-sweep.sh)，全文输出 [`02-corpus-sweep.out`](02-corpus-sweep.out)。
两二进制：父提交 `dccd21c3` 构建（`/tmp/rsgfi/bin/jarde-cli-base`，SHA-256 `dbbcd3e9…`）与修复后构建。
三档：A 单类姿态（2752 个 loose `.class`）、B 折叠姿态（2140 个 loose 根类，每个类按其自身声明的名字
拼装家族 jar）、C 每个已提交 jar 的每个 `.class` 条目（738）。

## 自检（先于任何计数）

```text
SELF-TEST OK: stated_name reads MN$Hold from the nested fixture and nothing from a non-class
SELF-TEST OK: MN family Holdava 2 -> 0; MN single-class byte-identical; bs.jar!BS.class byte-identical
```

- `stated_name` 自检（正例：嵌套类文件必须读出 `MN$Hold`；负例：非 class 文件必须读出空）；
- 已知正例：`MN` 家族折叠渲染 `Holdava` 2 → 0；
- 已知负例：同类的单类姿态渲染两二进制逐字节相同；无关 jar `bs.jar!BS.class` 逐字节相同；
- 结构性自检：**pass A（单类姿态，全语料）moved 必须为 0**，否则脚本以非零退出。

**脚本自身第一版的假零（如实记录）**：首版 `stated_name` 用
`grep -m1 -oE ' (class|interface|enum) [^ <]+'` 读名字，命中的是 `javap -v` 输出里 **InnerClasses 表的
注释行**（`// Hold=class MN$Hold of class MN`）而不是声明行，于是所有 `$` 命名兄弟文件都被读成根名、
家族 jar 里只剩根类、折叠被拒（`member_family.state = "refused"`）、两二进制输出同一份未折叠文本
——**pass B moved=0 的假零**。修正后（只读 `javap -v` 第一条非缩进声明行、取关键字后的 token，兼顾泛型
声明行 `<T extends …> extends …`）复跑得到下表。首版结果未采信，其 `stated_name` 自检现已写进脚本。

## 结果

```text
pass A: moved=0  unrendered=1
pass B: root classes=2140 moved=10 unrendered=2
pass C: moved=0  unrendered=1
moved classes: single-class=0 fold=10 jar=0 total=10
```

**单类姿态与 jar 档 0 差异**：本 change 只在家族折叠里生效，单类姿态全语料逐字节相同是预期不变量
（也是脚本的硬自检）。

## 10 个差异类逐类分类

| # | 类 | 形 | 父提交（逐字） | 修复后（逐字） |
| --- | --- | --- | --- | --- |
| 1 | `…/generic-static-field-init-patrol/fixture/MN.class`（巡查锚，javac23 腿） | 静态泛型字段 `Signature` 被拒 + 内联初始化 | `static Hold f1 = new MN$Holdava.lang.Object) "a");` | `static Hold f1 = new Hold((java.lang.Object) "a");` |
| 2 | 同目录 `RG.class`（巡查锚，嵌套形） | 同上 | `static Hold nested = new RG$Holdava.lang.Object) new RG$HolHold.lang.Object) java.lang.Integer.valueOf(5)));` | `static Hold nested = new Hold((java.lang.Object) new Hold((java.lang.Object) java.lang.Integer.valueOf(5)));` |
| 3-4 | `tests/fixtures/recover-static-generic-field-init-text/{v8,v8-javac8}/MN.class` | 同上（本片冻结的巡查 fixture，两条腿） | 同 #1 | 同 #1 |
| 5-6 | 同目录 `RG.class`（两条腿） | 同上 | 同 #2 | 同 #2 |
| 7-8 | 同目录 `SG.class`（两条腿） | 同上（本片可整类编译的同形锚） | `static Hold f1 = new SG$Holdava.lang.Object) "a");` | `static Hold f1 = new Hold((java.lang.Object) "a");` |
| 9-10 | 同目录 `P1.class`（两条腿） | **非泛型**静态字段 + 内联初始化（机制的对照） | `static Box b = new P1$Box;` | `static Box b = new Box(1);` |

机械分类：10 个差异类共 18 行 `>`/18 行 `<` 变更，**全部**是字段声明行（
`grep -A11 '^MOVED' … | grep -E '^\s+[<>]'` 中不属于字段声明的行数 = 0）；每类的池形名行数（非注释行含
`$`）都从 2 → 0。

**如实记录两点**：(1) 差异类**全部**是本 change 的锚与 fixture（巡查的两个锚 + 本片冻结的四类 × 两条腿），
语料中没有其它类落入该形；(2) 其中 `P1` 是**非泛型**形——机制（折叠重拼落点）与泛型无关，
`SG`/`MN`/`RG` 是静态泛型字段形。分类与 proposal 的"差异类只能是含静态泛型字段初始化的类"相比多出
`P1` 这一类非泛型差异，原因已在 [`01-mechanism.md`](01-mechanism.md) 1.1.5 说明（判别变量是"声明里被
折叠类名出现 ≥2 次"，非泛型同样触发）。

## 未渲染候选（4 条，逐条解释）

```text
A: tests/fixtures/proved-java-structure/package-info-basic/v8/p/package-info.class renders under no name it states
B: tests/fixtures/p4-modern/v17/module-info.class states no class name
B: tests/fixtures/proved-java-structure/package-info-basic/v8/p/package-info.class renders under no family name
C: openspec/evidence/java-syntax-2026-10-05/package-info-patrol/fixture/pi.jar!com/example/package-info.class renders under its own entry name
```

全部是 `package-info`/`module-info`：它们不是可呈现的类声明（`package-info` 是包声明载体，
`module-info` 声明 `module` 而非 class/interface/enum，`stated_name` 因此读空），且都**没有** `$` 成员，
折叠姿态对它们不适用。与本次改动无关，逐条列出而非静默跳过。
