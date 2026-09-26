# DT-10：普通枚举常量数量边界

此证据只覆盖普通顶级枚举声明：零、一、四个常量。三个枚举和 runner 都是独立顶级类；没有嵌套枚举，也没有匿名常量体，因此结果只用于 DT-10，不用于 DT-12 或 DT-13。Java 8 原始源码、全部四个 class 文件和 SHA-256 清单位于 [`tests/fixtures/proved-java-structure/enum-arity/`](../../../../tests/fixtures/proved-java-structure/enum-arity/)。

使用本地 JADX checkout `2fb1b163` 构建的 `jadx` 命令运行：

```sh
JADX=/path/to/jadx python3 openspec/evidence/java-syntax-2026-09-27/enum-arity/replay.py --expect-jarde baseline
```

脚本在临时目录用 `javac --release 8 -g:none` 编译原始源，核对冻结 class 的 SHA-256，并将同一 jar 分别交给 JADX 与 Jarde 的完整 class-source 路径。Jarde CLI 使用临时 `CARGO_TARGET_DIR`；退出时连同所有编译和中间文件一起清理。证据日志会把临时目录路径替换为 `<TMP>`。

`--expect-jarde fixed` 是后续修复的验收入口：它要求 Jarde 完整源码通过 Java 8 编译、`-Xverify:all` 执行并输出相同结果，修后源码和日志单独写入 `fixed/`，不会覆盖下方记录的失败基线。

原始源码和 JADX 的完整源码均通过 Java 8 重编及 `java -Xverify:all`，runner 输出相同：

```text
empty=0
one=ONLY:0/1
four=[NORTH, SOUTH, EAST, WEST]
```

Jarde 对四个 class 的 class-source 请求均成功，但三个 enum 文本都把物理字段作为普通成员写出，没有恢复源级枚举常量。完整源码集的 Java 8 重编失败；`javac` 分别在 `Empty.java` 的 `$VALUES`、`One.java` 的 `ONLY` 和 `Four.java` 的 `NORTH` 声明处报告 `enum constant expected here`。因此三种枚举都各自提供了确定的源码语法失败证据；Jarde 的完整源码集未能编译，故没有运行它的 runner，也不声称三种枚举的运行行为已验证。此结果只覆盖 DT-10，不外推到带匿名常量体的 DT-12。

测试工具版本及原始/两份反编译源码、编译日志、运行日志、Jarde CLI 状态和空枚举 `javap` 输出均保存在本目录。冻结输入由 JDK 23.0.1 的 `javac --release 8 -g:none` 生成。
