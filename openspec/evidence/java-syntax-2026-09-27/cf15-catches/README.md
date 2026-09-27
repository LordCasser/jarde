# CF-15 catch handler 审计（2026-09-27）

Jarde 基线：`761649d8aad021aca6b5df1a64f576dce0714aef`，分支 `codex/cf15-catch-audit`。固定 JADX checkout `/Users/lordcasser/workspace/testzone/jadx`，HEAD `2fb1b16386941660fda07e9017285aec40fcb37f`，工作区干净。

## 固定测试/实现哈希与断言强度

| 文件 | SHA-256 | 实际断言强度 |
|---|---|---|
| `TestMultiExceptionCatch.java` | `8aa6a88c376bbfa65db2530010a22f0cdda5ccfdbce3491a6f1be61fbaf578a2` | 验证一个 try、一个 `ProviderException | DateTimeException` catch、一个抛出 `RuntimeException(e)`，并拒绝错误的 `RuntimeException e;` 降级声明；包含结构和 catch 变量使用断言。 |
| `TestTryCatchMultiException.java` | `cc080954815e633f600b688e27e8ec5ec5027edc4fe41ce7729add46a2b8f47b` | no-debug 场景检查一个 multi-catch 头和 catch 变量引用；不验证完整 body 行为。 |
| `TestTryCatch7.java` | `acf63d45e4e10a447741fe84519605658a02ecade0b4b7f1217ce5ddeae1e67b` | 普通 `catch (Exception)` 的 catch 变量、写回外层变量、printStackTrace 与 return 在 debug/no-debug 输出中的结构断言；没有值语义 `check()`。 |
| `TestEmptyCatch.java` | `48e808573c32b7e099363b79a4db5dd078fe92348f43fe8e524459511e3748c3` | enum remap fixture 只数五个 try 和五个 `catch (NoSuchFieldError unused)`；不单独断言 handler body 真为空或各项映射正确。禁用编译。 |
| `TestUnreachableCatch.java` | `1257b614d67bbaf1f3286a3ceab3ef8d831b1826ab1344b7842c1fcbedf39efd` | 注释掉的 Java 片段不是测试输入；实际 Smali fixture 仅检查输出含 `IOException` 和 `Collections.unmodifiableMap`，没有断言 catch 结构或 handler 可达性。禁用编译并允许 warning。对应 Smali SHA-256：`6918e1253ac4582b199fee32b41214326204ea157c783a9e31142fc8d67091ad`。 |
| `ExcHandlersRegionMaker.java` | `bec06f3ebbd671a7a45a6949e3adb583cde365005fef84c3b5363302edb2279c` | 将异常表 handler row 映射到 handler region。 |
| `ProcessTryCatchRegions.java` | `0a382bc7e189742410b7a1b93f90703f60dfd7cc1256e06ddca4da0ccff6225d` | 根据 splitter、handler 可达边和 region child 收集/包装 try body。 |
| `TryCatchRegion.java` | `3efe1fdc13b6a34620ed4b93422ef816d6830338b1ea2dc978e16303568cc3b2` | 按 handler region 保存 catch 映射，finally 独立保存，普通条目以 `ExceptionHandler` 为 key。 |
| `RegionGen.java` | `8b80ef618487e38cb09a2bbd81b70de0dd096996a321a0c9dcf55e1dc011d4ee` | 遍历 handler map 输出 catch；同一 handler 的类型由 `getCatchTypes()` 用 ` | ` 连接。 |

这些 SHA 均从固定 JADX checkout 逐文件重算确认。`ExcHandlersRegionMaker` 会依据 handler 与 splitter 的路径交叉和 dominator/frontier 确定 region 出口；`ProcessTryCatchRegions` 在直接 child 中定位 top splitter，并排除可由 handler 到达、属于 try 结束之后的块。这里记录的是实现入口和必要条件，不推断所有 handler CFG 都覆盖。

## Java 8 三方首片

`input/CatchShapes.java` 同时覆盖顺序 typed catches 和 Java 8 multi-catch，catch 变量都被使用；main 依次运行无异常、两个普通异常和 multi-catch 的两个异常类型。完整原始、JADX、Jarde 类源码分别通过 `javac --release 8 -g -Xlint:-options`，并以 `java -Xverify:all` 执行。五行逐字相同：

```
ok
argument:arg
state:state
IllegalArgumentException:multi
IllegalStateException:multi
```

`javap -c -v` 的异常表也确认了类型顺序和 handler 关系：`ordinary` 的 BCI 范围 `[0,32)` 先映射至 handler BCI 33 `IllegalArgumentException`，再映射至 BCI 57 `IllegalStateException`；`multi` 的 `[0,24)` 有两行，分别为 `IllegalArgumentException` 与 `IllegalStateException`，都指向 BCI 24。同一 handler 被还原为一个 multi-catch，运行时两种实际异常的类名输出未交换或丢失。

本轮没有发现普通 typed catch / multi-catch 首片差距。JADX 固定的 empty-catch 与 unreachable-catch 代表测试断言弱、且后者依赖 Smali；这两类没有纳入三方运行首片，仍需单独扩验，因此 CF-15 只标部分已测。

证据文件保存输入、三方完整源码、原始 class 和三方运行输出。定向验证结果：`CARGO_TARGET_DIR=/tmp/jarde-cf15-target CARGO_INCREMENTAL=0 cargo test --test p3_typed_catch --test p3_nested_try`，typed catch 6/6、nested try 2/2 通过；这些现有测试是补充，不代替固定 JADX 测试或本次三方源样本。

主线复核：root 用合入 CF-06/CF-08 的 CLI（SHA-256 `95295d3e1688077b9cde0620739178525fb95b77084425540452b79623b791d1`）重新恢复同一原 class，完整源码 SHA-256 为 `4be2fb3a699f16bba03336b882afedf88476940877b9345e0b6ea0b201286d3f`，与本目录 Jarde 归档相同；再次 Java 8 重编并 `-Xverify:all` 运行输出同上述五行。原始/JADX/Jarde 三份归档 class 亦独立验证运行一致。
