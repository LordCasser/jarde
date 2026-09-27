## Why

当前已证明的枚举匿名常量体只覆盖零显式构造参数；JADX 的 `TestEnums2a` 以两个匿名常量体和一个 `String` 常量实参展示了相邻源形态，而 Jarde 保留物理字段与访问构造器，导致完整输出不能按 Java 8 重编。

## What Changes

- 增加一个有界 Java 8 源码投影切片：准确的双常量顺序、每个常量一个已证明可无损拼写的 ASCII 字符串字面量实参、共享单 `String` 源构造器及已支持的无捕获匿名体。
- 仅在同一封闭证据计划证明常量顺序、两个匿名类体、字符串实参与构造器/访问桥的 name、ordinal 转发关系后，原子输出枚举常量列表和体；任一条件、执行范围或预算不完整时拒绝这一联合投影并保留物理来源。
- 对正例和构造桥/实参关系负例做原版、JADX、Jarde 完整源码 Java 8 重编及 `-Xverify:all` 行为对照。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 为已证明的双匿名常量体枚举增加一个字符串构造实参的安全源码投影契约。

## Impact

影响 `jarde` 的 class-source 同次类事实/枚举候选与匿名体证明，以及 Java 8 来源文本投影；不改变 reader 公共 classfile 模型，不引入新依赖，不扩展 query、CLI 或运行目标代码的能力。此提案来自 [DT-12 冻结审计](../../evidence/java-syntax-2026-09-27/dt12-anonymous-enum-audit/analysis.md)。该提案只覆盖 `TestEnums2a` 所代表的双常量单 ASCII `String` 字面量输入，不覆盖 `TestEnums6`（它没有匿名常量体）、DT-11 String varargs、非 ASCII 字面量、任意类型/表达式实参、其它常量数量、多个构造器、捕获或任意桥接形态。
