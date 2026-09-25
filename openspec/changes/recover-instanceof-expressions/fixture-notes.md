# 夹具记录

本轮只修改 `tests/p3_instanceof.rs`、本记录并新增
`openspec/evidence/java-syntax-2026-09-22/instanceof/refusal-boundaries/`，未修改生产代码、
`tasks.md`、census、fingerprint 或 README 总表。永久语料只有
`tests/fixtures/p3-instanceof/v8/InstanceOfProbe.class`；`InstanceOfSupport.java` 和
`InstanceOfRunner.java` 仅作为 source-only 的 JDK 输入。

## 固定 class 与原始执行

使用 `javac 23.0.1` 执行：

```text
javac --release 8 -g:none -d /tmp/jarde-instanceof-20260923 \
  InstanceOfSupport.java InstanceOfProbe.java InstanceOfRunner.java
```

冻结 class 为 Java 8（major 52），1548 字节，SHA-256 为
`d8f4a437d0cd0f3ec12672791f27b1c11cfbcdc6885d10eddcd3114aa0838e56`。
`javap -v -c` 显示 15 个带 `Code` 的方法、0 个字段：构造器、12 个公开测试方法以及
`keep`、`empty` 两个私有辅助方法。`widenedString` 的 `aload; instanceof Integer` 之间没有
`checkcast`；`called` 的 `InstanceOfSupport.value():String` 之后也没有 `checkcast`；
`functional` 是 `invokedynamic ... ()Ljava/lang/Runnable;` 后紧接 `instanceof Runnable`。
class 使用 `-g:none`，没有局部变量调试表。

`java -Xverify:all` 运行冻结 class 的 source-only runner 输出：

```text
string-text=true
string-null=false
string-number=false
null=false
runnable=true
runnable-null=false
primitive-array=true
primitive-array-string=false
reference-array=true
reference-array-object=false
multi-array=true
multi-array-one=false
widened-string=false
local-true=true
local-false=false
parameter-true=true
parameter-false=false
branch-true=1
branch-false=0
functional=true
called=false:1
called-fail=java.lang.IllegalStateException:1
```

## 正面边界

正面类覆盖 Object 与 `null` 的类/接口测试、`int[]`/`String[]`/`String[][]`，`String` 经
`Object` 安全加宽后测试 `Integer`，返回 `String` 的有计数调用经 `Object` 测试及 producer
自身异常，真实 `Runnable` method reference 以及 boolean 的局部、调用参数和普通 `if` 消费。
0/1 汇合、丢弃结果、重复消费和 boolean 整数算术等负面形状没有放入该可整类编译的正面
语料。

## 1.2 负面消费边界

`refusal-boundaries/` 以冻结 class 的完整 `method_info` 精确替换收录四个合法 JVM 变体，
没有新增永久 class，也没有通用 class-file parser。`pop` 把 `called()Z` 改为
`called()V`，保留 BCI 0 的有计数 producer、BCI 3 的 `instanceof` 和 BCI 6 的 `pop`；
`duplicate` 在 `local(Object)Z` 的 BCI 4 加 `dup`，由 BCI 5/6 两个 local store 消费；
`stale` 在 BCI 4 的第一次 store 后于 BCI 5/6 写入 `false` 覆盖同一 local，再由 BCI 7 读取；
`int` 将 `branch(Object)I` 改为 BCI 1 `instanceof`、BCI 4 `iconst_1`、BCI 5 `iand`、
BCI 6 `ireturn`，并移除旧的 `StackMapTable`。四者分别为 1549、1550、1550、1535 字节，
SHA-256 与逐条 `javap -p -c -v` 输出见该目录的 `class-sha256.txt`、`*-javap.txt` 和
`patch-stats.txt`。

四个变体均通过 `java -Xverify:all`。source-only `RefusalRunner` 记录了 `pop` producer
只调用一次、duplicate 保持 true/false、stale 两个输入均为 false，以及 int consumer 的
1/0/0。测试通过 all-evidence class-source 入口要求真实 producer、测试和消费者 BCI 映射到
物理 member；若部分形状暂不能呈现，必须保留必要 quoted BCI 和来源，不能输出非法
`instanceof` 独立语句、旧 local 值或 boolean-to-int `iand` Java 表达式。

当前未重建的 `target/debug/jarde-cli` 对四个变体均返回 0，但完整生成源的
`javac --release 8 -g:none` 仍失败（正面 class 的既存缺口也保留在同一完整请求中）；原始
CLI 文本与编译日志在 `refusal-boundaries/jarde/`，不是对变体行为的运行时 oracle。

## 修前红证据

使用当前已有的 `target/debug/jarde-cli`（未重建）执行：

```text
target/debug/jarde-cli class-source \
  --input tests/fixtures/p3-instanceof/v8/InstanceOfProbe.class \
  --class InstanceOfProbe --policy single-class --release 8 --format text \
  --evidence all --output <temporary>/InstanceOfProbe.java
```

将完整输出和 source-only helper/runner 用 `javac --release 8 -g:none` 编译，退出码为 1，报告
12 个 `缺少返回语句`。当前恢复输出对所有真实 `instanceof` 方法保留了 bytecode 说明而没有
返回表达式；`called` 只保留了副作用调用并丢失测试结果，`functional` 也保留了未消费的
lambda 说明。该结果是本 change 的修前红证据，不能通过删除这些方法来换取整类通过。
JADX 的完整输出及其中三处 Java 8 编译错误由 root 的独立 baseline 保存；本记录不把 JADX
误报为可运行对照。

## Rust 回归边界

`tests/p3_instanceof.rs` 通过 `Engine::class_source_with_evidence` 的完整类入口，逐一要求 15
个方法为 `Recovered`、`Java`、`Structured` 且包含语句。它检查目标类型、Object 安全加宽、
单次 producer、method reference 和三种 boolean 消费方式。`essential()` 与 `all()` 的完整
正文必须逐字相同；默认请求的 source map 为空，`all()` 每个 segment 都必须有非空 BCI 和
物理 member。ignored JDK 测试先编译 helper/runner，再将冻结 class 覆盖回 original 目录，
随后编译完整 Engine 输出，双方以 `-Xverify:all` 运行并比较全部输出，不替换或删除生成方法。

本轮未运行 Cargo；只执行了 `javac`、`java`、`javap`、`rustfmt --edition 2024` 和
`git diff --check`。

## Root独立边界复核

Root从当前Rust测试的精确字节数组重新生成变体，在独立目录复编译helper/runner并再次执行`-Xverify:all`，结果保存于`refusal-boundaries/root/`。原四个变体的hash与报告一致。原`stale`只是读取已覆盖的新局部值，并没有让旧值留在栈上；root另加`retained_old_local_patch`：BCI 5先读取旧boolean，BCI 7写入false，BCI 8返回栈上旧值。该1550-byte合法class的两行结果为true/false，SHA256为231d22ac22157f83160ff2d7b88d4596238930f288620217400b22645d753f28。旧值不能被新的false替换；如未来共享值保存实际恢复，应保留正确运行结果。

根测试同时移除普通覆盖变体对`return false`的错误一律禁止：若测试本身仍被表达且来源完整，false正是这个变体应返回的值。类长度断言的名字只声称检查长度，JVM合法性由独立执行记录证明。生产和Rust行为验收仍待串行实施，未以这些修前证据宣称instanceof已恢复。
