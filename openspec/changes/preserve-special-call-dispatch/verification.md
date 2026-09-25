# Verification

## Scope

本文件记录任务 1.1 的永久 fixture、修复前基线和当前修复验证。源码位于
`tests/fixtures/p3-special-dispatch/`，由 `javac 23.0.1` 使用 `--release 8 -g:none` 生成
三个被恢复的 class。driver 只提交 `.java`，执行时在临时目录编译。回归测试是
`tests/p3_special_dispatch.rs` 与 `tests/p3_special_refusal.rs`；普通断言固定修复后的文本，JDK
运行断言标记为 ignored，仍会在显式运行时执行真实恢复文本。

## Commands

在仓库根目录可重放：

```text
javac --release 8 -g:none -d tests/fixtures/p3-special-dispatch/v8 \
  tests/fixtures/p3-special-dispatch/BaseProbe.java \
  tests/fixtures/p3-special-dispatch/DefaultProbe.java \
  tests/fixtures/p3-special-dispatch/SpecialProbe.java
mkdir -p /tmp/special-runner
javac --release 8 -g:none -cp tests/fixtures/p3-special-dispatch/v8 -d /tmp/special-runner \
  tests/fixtures/p3-special-dispatch/SpecialRunner.java
java -cp tests/fixtures/p3-special-dispatch/v8:/tmp/special-runner SpecialRunner
jar cf /tmp/jarde-special-dispatch.jar \
  -C tests/fixtures/p3-special-dispatch/v8 BaseProbe.class \
  -C tests/fixtures/p3-special-dispatch/v8 DefaultProbe.class \
  -C tests/fixtures/p3-special-dispatch/v8 SpecialProbe.class
/Users/lordcasser/workspace/projects/jarde/target/debug/jarde-cli class-source \
  --input /tmp/jarde-special-dispatch.jar --class SpecialProbe \
  --policy plain-jar --release 8 --format text
/opt/homebrew/bin/jadx -d /tmp/jarde-special-dispatch-jadx --no-res \
  /tmp/jarde-special-dispatch.jar
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 \
  cargo test --test p3_special_dispatch --locked
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 \
  cargo test --test p3_special_refusal --locked
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 \
  cargo test -p jarde-jvm --locked method_ir::tests
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 \
  cargo test --test p3_special_dispatch --locked \
  generated_special_dispatch_matches_original_runtime -- --ignored --nocapture
```

## Source and JVM evidence

`SpecialProbe.java` 的关键源码为：

```java
public int value() {
    return super.value() + 1;
}

public int defaultCall() {
    return DefaultProbe.super.value();
}

public int callOwnPrivate(int value) {
    return privateHelper(value);
}

public int callOtherPrivate(SpecialProbe other, int value) {
    return other.privateHelper(value);
}

public int superWithSideEffect() {
    return super.valueWith(BaseProbe.sideEffectArgument());
}

public int superWithThrowingArgument() {
    return super.valueWith(BaseProbe.throwingArgument());
}

public int superThrowing() {
    return super.failWith(3);
}
```

`javap -p -c -s tests/fixtures/p3-special-dispatch/v8/SpecialProbe.class` 的关键 owner/receiver 为：

```text
value: aload_0; invokespecial BaseProbe.value:()I; iconst_1; iadd; ireturn
defaultCall: aload_0; invokespecial InterfaceMethod DefaultProbe.value:()I; ireturn
callOwnPrivate: aload_0; iload_1; invokespecial privateHelper:(I)I; ireturn
callOtherPrivate: aload_1; iload_2; invokespecial privateHelper:(I)I; ireturn
superWithSideEffect: aload_0; invokestatic BaseProbe.sideEffectArgument:()I;
  invokespecial BaseProbe.valueWith:(I)I; ireturn
```

原 class driver 输出：

```text
value=8
defaultCall=11
own=7
other=8
otherNull=java.lang.NullPointerException
superSideEffect=8
sideEffectCount=1
superThrowingArgument=java.lang.IllegalArgumentException
superThrowingParent=java.lang.IllegalStateException
```

## Three-way baseline

jadx 1.5.6（同一 jar）摘录：

```java
public int value() {
    return super.value() + 1;
}

public int defaultCall() {
    return super.value();
}

public int callOtherPrivate(SpecialProbe specialProbe, int i) {
    return specialProbe.privateHelper(i);
}
```

jadx 保留了 class-super 和其它实例 private receiver，但把 `DefaultProbe.super.value()` 降成
`super.value()`；与同一父类/接口编译后 `defaultCall` 返回 7，原 class 返回 11。

修复前 debug jarde `/Users/lordcasser/workspace/projects/jarde/target/debug/jarde-cli` 摘录：

```java
public int value() {
    return this.value() + 1;
}

public int defaultCall() {
    return this.value();
}

public int callOwnPrivate(int arg1) {
    return this.privateHelper(arg1);
}

public int callOtherPrivate(SpecialProbe arg1, int arg2) {
    return arg1.privateHelper(arg2);
}

public int superWithSideEffect() {
    return this.valueWith(BaseProbe.sideEffectArgument());
}
```

从该真实 class-source 文本写回 `SpecialProbe.java`，再以 fixture 的父类、接口和 driver 执行，编译
成功但输出为：

```text
value=java.lang.StackOverflowError
defaultCall=java.lang.StackOverflowError
own=7
other=8
otherNull=java.lang.NullPointerException
superSideEffect=8
sideEffectCount=1
superThrowingArgument=java.lang.IllegalArgumentException
superThrowingParent=java.lang.IllegalStateException
```

因此修复前错误是可执行的 self-recursive dispatch，而不是按方法名推测的风险；`other` 接收者、null
NPE、参数副作用和参数/父类异常仍可观察。修复后的测试 SHALL 保持原 class 的 8/11 以及 `this`/其它
实例 receiver；它 MUST 从库生成的文本编译，不得用替代方法体掩盖恢复错误。

## Refusal and read budget

`special_receiver` 只有在同源类头给出直接父类/接口、池项类型相符且 SSA receiver 是入口
`Local(0)` 时才构造 `super`。同类目标的 private 判定优先使用同一类头的精确
name/descriptor/flags；只有调用方没有交出该类头时，才使用 owner 相同的现有 `ClassMembers`
声明，类头中的明确 public 不会被另一份视图覆盖。缺少任一证据、非入口 `this` 或同类非 private
目标均沿用原有 fallback。构造器路径没有进入这项选择。

该选择只借用 `MethodIr` 已持有的 `Arc<ClassFacts>`：池项的 `MethodRef`/`InterfaceMethodRef`、
直接父类、直接接口和成员头都来自同一次读取；它不解析祖先、不读取被调方法体，也不启动第二个
class header。拒绝的调用仍通过 `quoted_bcis` 递归保留延期 invocation 及其 receiver/参数生产者，
并沿用值深度上限与已存在的 BCI 去重。

独立 class-source 对照 `/tmp/jarde-dispatch-independent/jarde-after.report.txt` 的 usage 记录为
`read_bytes = 591`、`class_headers = 1`、`method_bodies = 8`；这是该 class 的一次类头读取和八个
方法体读取。各方法报告继续携带同一次 header 事实，没有为 special receiver 另读类头或 body。

## Test status

生产修复后的 debug 构建通过：

```text
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo check -p jarde-java --locked
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test --test p3_special_dispatch --locked
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test --test p3_special_dispatch --locked -- --ignored --nocapture
```

专项结果为普通 3/3、拒绝边界 4/4、MethodIr seam 2/2 通过，JDK ignored 1/1 通过；恢复文本由
`Engine::class_source` 实际生成后编译执行，输出保持原 class 的 8/11、private receiver、null NPE、
参数副作用与两类异常。拒绝边界测试还确认非入口 receiver、同类 public special，以及返回/丢弃结果两种消费形状的
延期生产者的 fallback/source-map 保留。第四个拒绝样例实际保持 `()I`，使用
`pop; iconst_0; ireturn` 丢弃 special 结果；它不是 void 方法，未修改共享 descriptor 池项。

## 主代理最终复核

- `/tmp/jarde-special-final-integration.log`：special 正面 3 项、拒绝 4 项、旧局部值 7 项、失败区域 3 项，共 17 passed / 1 ignored。
- `/tmp/jarde-special-final-java.log`：Java 包 93 个库测试、32 个 recovery 测试、43 个 pattern 测试，共 168 项通过。
- `/tmp/jarde-special-final-jdk.log`：从最终生产代码生成的完整 SpecialProbe 文本再次编译执行，1 项 JDK 对照通过。
- `/tmp/jarde-special-final-seam.log`：MethodIr 的 2 项 seam 测试通过，包括无类头时不给出 special 证据。编译提示既存 canonical 测试函数未使用，未在本改动中修整该区域。
- 主代理另造的带包名、void 两种 super、参数副作用和其它实例 private 样例共 7 个结果一致，实际 CLI 整类输出直接编译，证据在 `../../evidence/java-syntax-2026-09-22/special/independent/`。
- 重建 CLI 后，合法 JVM 拒绝反例由 `@bytecode 7 4` 变为 `@bytecode 7 4 1`，保留嵌套实参调用；修前/修后实际输出在 `special/refused-producer/`，无需回滚源码伪造红测。
- `cargo fmt --all -- --check` 和 `git diff --check` 均通过；OpenSpec 全量 strict 为 29 passed。
- 最终严格 clippy（jarde-java/jarde-jvm、all-targets、`-D warnings`）仍被既有 `region.rs:1736` 的 `type_complexity` 阻断，日志 `/tmp/jarde-special-final-clippy.log`。未添加 allow 或混入区域重构。因此 3.2 保留未勾选，不能把专项语义验收说成全仓门禁全部通过。
- fixture census 为 `(86,431,74,197,8)`，指纹普通检查 5 passed / 1 ignored；与本轮其它 fixture 一次集中核实，没有额外落盘 class。
