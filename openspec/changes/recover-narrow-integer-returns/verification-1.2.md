# Task 1.2：返回消费边界证据

本文件记录任务 1.2 的永久 Java 8 输入、实际 return/operand BCI、原 JVM 结果及 Jarde
边界报告。窄返回主 fixture 沿用现有 `NarrowIntegerReturns.class`；新增独立 Rust 回归位于
`tests/p3_narrow_return_boundaries.rs`，没有修改生产 crate 或 `tasks.md`。

## 字段与同步返回

`tests/fixtures/p3-narrow-integer-returns/v8/NarrowIntegerReturns.class` 是 1,288 字节，
SHA-256 为
`90219c1ac7c93b53bee50da43ac792cd7e628dba4bff7b0f697a8b64a9b5d14c`。它以精确 descriptor
patch 将 `int` 源方法改为 B/C/S，Code 属性不变。`javap -c -p` 的实际栈值生产与返回位置为：

| 方法 | 返回值生产 | 字段写入/监视器退出 | 实际 `ireturn` |
| --- | ---: | ---: | ---: |
| `postByte(I)B`、`postShort(F)S` | `getfield` BCI 2；`dup_x1` 保留旧值于 BCI 5 | `putfield` BCI 8 | BCI 11 |
| `preChar(J)C` | `iadd` BCI 6；`dup_x1` 保留和于 BCI 7 | `putfield` BCI 8 | BCI 11 |
| `syncByte(Object,I)B`、`syncChar(String,I)C` | 参数 load BCI 4 | `monitorexit` BCI 6 | BCI 7 |
| `syncShort(Object,J,I)S` | 参数 load BCI 5 | `monitorexit` BCI 8 | BCI 9 |

新增 `tests/p3_narrow_return_boundaries.rs` 还直接对窄返回主类的 post/pre 与同步方法断言
operand、putfield/monitorexit 和 ireturn 的来源；现有 `tests/p3_field_increment.rs` 对
`putfield` BCI 8 与 `ireturn` BCI 11 的直接/派生映射作断言，`tests/p3_sync_return.rs` 固定
同步块中的返回和对应指令映射。永久 runner 执行
49 行边界用例，覆盖更新前后字段值、B/C/S 越界返回及空锁抛出的
`java.lang.NullPointerException`。新增测试也执行该 runner，并断言 49 行与空锁异常存在。

## Java 8 真实共享栈 join

`tests/fixtures/p3-narrow-integer-returns/actual-stack-join/ActualStackJoin.java` 是合法的
Java 8 局部变量版输入。重放脚本以受控 stack rewrite 移除 switch 臂的局部 store 与 join
load，保留臂值在操作数栈上，并生成 major 49 类；然后用既有 descriptor patcher 单独改动
一个返回描述符。三份永久 class 都是 379 字节，Code 与 stack rewrite 后的基类逐字节一致：

| 方法/描述符 | B/C/S class SHA-256 | 唯一共享 `ireturn` BCI | case/default 值 BCI |
| --- | --- | ---: | --- |
| `runByte(I)B` | `ad9b88a41ec56ed7a5ff553388dcf3867f8d00bc5a6a27404da4976b7b684149` | 32 | 20 / 26 |
| `runChar(I)C` | `c5c2ab16469319970488a56cb881cb14873a23c55e59e77b4dd582f1dd3b7abe` | 30 | 20 / 25 |
| `runShort(I)S` | `a5f141e48dc0ee777e02d9cd4c87c25a27ca3b9b40f86909f8f05e2153196dd3` | 30 | 20 / 25 |

新增测试对各方法的 B/C/S descriptor、Java/Structured 结果、共享 `ireturn` 的直接 source
map、两臂值和 BCI 1 switch 均作断言；另外将三份 class 各自交给
`java -Xverify:all`，并逐字比较完整 runner 输出与 fixture 的 `expected.txt`。原 JVM 的
窄化结果包含 B 的 `130 -> -126`、`-129 -> 127`，C 的 `-2 -> 65534`，以及 S 的
`32768 -> -32768`、`-32769 -> 32767`。

重建命令为：

```sh
python3 tests/fixtures/p3-narrow-integer-returns/regenerate_task12_fixtures.py /tmp/task12-fixtures
```

脚本使用 `javac --release 8 -g:none`，并复用 OpenSpec evidence 中的
`actual-stack-join/patch_stack_join.py`、`patch_descriptors.py`。新生成的三份 B/C/S class、
boolean boundary class 与 raw-2 caller class 均与永久 fixture 按字节相同。

## Z、boolean-to-B/C/S 与 raw-2 边界

`v8/boolean-boundaries/BooleanReturnBoundaries.class` 的 `booleanAsByte(Z)B`、
`booleanAsChar(Z)C`、`booleanAsShort(Z)S` 保留 Z 参数并将返回描述符改为 B/C/S；方法 Code
仍是 `iload_0; ireturn`。三个方法的 JVM 验证均通过，`false/true` 返回分别为
`0/1`。Jarde 对各方法均返回 `Mixed/Fallback`，拒绝文本明确说明 BCI 0 的值由参数描述符
呈现为 boolean，而返回位置声明 byte/char/short，且缺少可证明的转换；BCI 1 的 `ireturn`
仍保留在 source map 中。

`BooleanRawTwoCaller.class` 的三个方法由 Java 8 编译器针对 `(Z)B/C/S` 目标生成，随后只在
调用者 BCI 0 将 `iconst_0` 替换为 `iconst_2`。这是 verifier-valid 的原 JVM 调用，三个返回
值都为原始整数 `2`。Jarde 在调用点拒绝该 `int` 操作数到 boolean 参数的未证明转换；
报告保留 `@bytecode 4 1`，并说明调用 BCI 1 的参数 0 声明为 boolean、当前 operand 呈现为
int。它不会把返回值改写成 `1`。该输入固定了 Java boolean 条件呈现与 JVM descriptor 执行
语义不同的边界。

同一 fixture 的 `integerAsBoolean(I)Z` 由 int identity 方法只改返回描述符得到。通过
`java -Xverify:all` 观察到输入 `0, 1, 2, 3, -1` 的 Java boolean 结果依次为
`false, true, false, true, true`，与最低位语义相符。这个 Z 返回转换目前保留为单独的
恢复债务；不能从条件返回的 true/false 文本推断一般 Z 返回都只需普通 Java boolean。
返回值条件 stack phi 仍是边界，不计作本项正例。

## 验证结果

- `CARGO_TARGET_DIR=/tmp/jarde-task12-target cargo test --test p3_narrow_return_boundaries`：3 项通过；测试字段/同步窄返回映射，真实执行三份 major 49 B/C/S join class、boolean boundary class、raw-2 caller 与原 49 行窄返回 runner。
- `CARGO_TARGET_DIR=/tmp/jarde-task12-target cargo test --test p3_narrow_integer_returns`：2 项通过，完整类执行对比测试保持 ignored（该用例属于后续整类验收）。
- `regenerate_task12_fixtures.py`：成功重建五个永久 class；B/C/S join 输入各 379 字节，全部与仓库 fixture 字节相同。
- private Cargo target 由 root 在最终验收后清理。
