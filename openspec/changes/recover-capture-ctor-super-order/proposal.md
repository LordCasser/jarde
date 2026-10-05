## Why

[双括号初始化巡查](../../evidence/java-syntax-2026-10-05/double-brace-patrol/README.md)实证（呈现缺陷级）：捕获形匿名伴生（`new ArrayList<String>(){{ add(s); }}` 捕获 `s`）的 ctor 渲染把 `this.val$s = arg1;` 放在 `super();` **之前**——这是 javac8 的字节码发射次序（`aload_0; aload_1; putfield val$s; aload_0; invokespecial <init>`），字节码层合法，但**标准 Java 禁止 super 调用前引用 this**（灵活构造器是 Java 21+ preview）。拼接渲染源 `javac` exit 1（"灵活构造器是预览功能"）。

**判别（完整）**：无捕获形（`DB$1`）健康——super 后内联合法；仅**捕获形**（有 val$ 字段赋值先于 super）损坏。**jadx 有解**：识别整个模式（匿名子类+纯实例块体+单分配点）在分配点直接呈现源级双括号形 `new ArrayList<String>() {{ add(str); }}`——绕开 ctor 排序问题。



> **root 追加锚（2026-10-05，[thread-anonymous 巡查](../../evidence/java-syntax-2026-10-05/thread-anonymous-patrol/README.md)）**：**Thread 匿名形 + 同形异序**——匿名 Thread 子类伴生（无尾随字段初始化）保持非法原始序（`this$0 = arg; super();` 编译失败），而字节码**逐条相同**的匿名 Runnable 伴生（有尾随 `local = 0`）被合法重排（`super(); this$0 = arg; local = 0;`）——重排路径仅在有后续初始化时触发；实现时统一两形。

## What Changes

**MVP（路径 A——伴生 ctor 重排）**：当伴生 ctor 的 **pre-super 语句全部是捕获字段赋值**（`this.val$x = argN`）且**其赋的值不流入 super() 调用的实参**（数据流可证）时，把渲染重排为 `super(); this.val$x = argN; …实例块语句…`——重排在"super 实参不依赖捕获值"的前提下语义等价（super 只是不读这些字段；javac8 之所以前置仅因需要，非语义必需）。不可证时**保持现状**（响亮——渲染头本就声明 not claimed to compile）。

路径 B（jadx 式分配点双括号形）是更彻底的源级还原，但需要匿名类模式识别+单点使用证明——留后续片（本片 Non-Goal）。

## Impact

- **代码**：伴生 ctor 渲染区（`member_inner.rs`/`facade.rs` 的伴生构造器呈现——val$ 赋值与 super 的次序逻辑，task 1.1 定位）。
- **测试**：`DB` fixture（双形判别已冻结）+ 无捕获形零回退 + 负例（super 实参依赖捕获值——如 `outer.new`/捕获值传 super——保持现状）。
- **账本**：summary.md 登记行关闭。

## Non-Goals

- **不**做分配点双括号形还原（路径 B——后续独立片，需模式识别）；
- **不**动无捕获形伴生（零回退锚）；
- **不**处理 super 实参依赖捕获值的形（保持现状响亮）。
