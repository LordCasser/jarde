# SpecialProbe：super/default/private 调度探针

日期：2026-09-22（Asia/Shanghai）。这是独立的自创 Java 8 fixture，没有修改工作区代码。

## 输入与命令

源码在 [src/](/tmp/jarde-special-probe-20260922/src/)；使用 `javac --release 8 -g:none` 编译 `BaseProbe`、`DefaultProbe`、`SpecialProbe`、`SpecialRunner` 和仅用于异常类型检查的 `GeneratedRunner`。`SpecialProbe.class` 与父类/接口一起打包到 [special-probe.jar](/tmp/jarde-special-probe-20260922/special-probe.jar)。原始字节码见 [javap-SpecialProbe.txt](/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-09-22/special/javap-SpecialProbe.txt)。

jarde 主结果使用较新的 `/Users/lordcasser/workspace/projects/jarde/target/debug/jarde-cli`，分别对同一 jar（`plain-jar`）和同一 `SpecialProbe.class`（`single-class`）运行 `class-source` text/JSON；两份 text 完全相同。结果在 [jarde-jar-text.txt](/tmp/jarde-special-probe-20260922/jarde-jar-text.txt)、[jarde-jar.json](/tmp/jarde-special-probe-20260922/jarde-jar.json)、[jarde-class-text.txt](/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-09-22/special/jarde-class-text.txt)、[jarde-class.json](/tmp/jarde-special-probe-20260922/jarde-class.json)。jadx 1.5.6 使用同一 jar，源码在 [jadx/sources/defpackage/SpecialProbe.java](/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-09-22/special/jadx.java.txt)。可重放命令见 [replay.sh](/tmp/jarde-special-probe-20260922/replay.sh)。

## 运行时事实与字节码事实

原始 fixture 的 [runner.stdout](/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-09-22/special/runner.stdout) 是：

```text
value=8
defaultCall=11
own=7
other=7
```

这四个值由源代码和 JVM 运行共同确认：`BaseProbe.value()` 返回 7，`DefaultProbe.value()` 返回 11；覆写方法调用 `super.value()+1` 得 8，显式接口 default 调用得 11；两个 private helper 调用都得 7。

`javap` 显示的关键 owner 是：`value()` 在 BCI 1 使用 `invokespecial BaseProbe.value:()I`；`defaultCall()` 在 BCI 1 使用 `invokespecial InterfaceMethod DefaultProbe.value:()I`；两个 private helper 调用都使用 `invokespecial privateHelper:(I)I`。因此 `value()` 的 `super` 与 `defaultCall()` 的 `DefaultProbe.super` 在字节码中是可区分的事实。

## 三方逐方法对照

| 方法 | 源码 / 原始运行意义 | jadx 1.5.6 | jarde debug（jar 与 class 相同） |
|---|---|---|---|
| `<init>()V` | 隐式 `super()`，调用 `BaseProbe.<init>` | 构造器省略 | `structured/java`，输出 `super(); return;`；`jre_constructor_prologue` 明确 owner 为 `BaseProbe`，无自递归。 |
| `value()I` | `return super.value() + 1;`，运行值 8 | `return super.value() + 1;`，保持 BaseProbe owner | `structured/java`，错误输出 `return this.value() + 1;`。若把 jarde 文本单独编译，调用 `value()` 得 `java.lang.StackOverflowError`，见 [generated-runner.stdout](/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-09-22/special/generated-runner.stdout)。这是已实测的 self-recursive dispatch 错误，不是根据命名推断。 |
| `defaultCall()I` | `return DefaultProbe.super.value();`，运行值 11 | `return super.value();`；在同一 BaseProbe/DefaultProbe 类关系下编译后得 7，见 [jadx-generated-runner.stdout](/Users/lordcasser/workspace/projects/jarde/openspec/evidence/java-syntax-2026-09-22/special/jadx-generated-runner.stdout)，丢失接口 default 选择 | `structured/java`，错误输出 `return this.value();`，再次落到 SpecialProbe 覆写并触发 `StackOverflowError`。 |
| `privateHelper(I)I` | `return value + 2;` | `return i + 2;` | `structured/java`，`return arg1 + 2;`；参数重命名但值语义一致。 |
| `callOwnPrivate(I)I` | `return privateHelper(value);`，运行值 7 | `return privateHelper(i);` | `structured/java`，`return this.privateHelper(arg1);`；值语义一致。 |
| `callOtherPrivate(LSpecialProbe;I)I` | `return other.privateHelper(value);`，运行值 7 | `return specialProbe.privateHelper(i);` | `structured/java`，`return arg1.privateHelper(arg2);`；接收者保留，值语义一致。 |

## 结论

这个 probe 复现了两个真实错误：jarde 把 `invokespecial BaseProbe.value` 错写成当前类 `this.value()`，造成确定的自递归；把 `invokespecial DefaultProbe.value` 错写成当前类 `this.value()`，同样造成自递归。jadx 保留了 `value()` 的 `super`，但将 `DefaultProbe.super.value()` 降成 `super.value()`；其反编译文本可编译，却把运行值从 11 改成 7。private helper 的本类接收者与另一 `SpecialProbe` 接收者在 jarde 中均保持正确。

jarde JSON 对 jar/class 两种输入均报告 6 个方法 `structured/java`、`verification=not_performed`；“structured”只表示体形状完整，不代表 owner/dispatch 语义已验证。`GeneratedRunner` 的结果只针对 jarde/jadx 生成文本的可编译副本，未运行任何外部目标。
