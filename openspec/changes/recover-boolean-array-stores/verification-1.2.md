# Task 1.2 fixture reproduction and stage record

本记录仅完成任务 1.2。永久 fixture 位于 `tests/fixtures/p3-boolean-array-stores/`；测试从已提交补丁 class 运行 oracle，并将 source-only runner 与完整 Jarde 类分别写入系统临时目录编译。测试运行创建的 class 文件不会进入仓库。

## 原类与受控补丁

Java 源码由 OpenJDK/Javac 23.0.1 以 `javac --release 8 -g:none` 编译，产生 classfile major version 52。复现时所有中间文件写入私有目录 `/tmp/jarde-bastore-fixture-output`，使用已有审计脚本 `openspec/evidence/java-syntax-2026-09-25/boolean-array-lowbit/patch_classfiles.py`。该脚本只改同长度 descriptor UTF-8 项及目标方法 Code 内的两条 opcode；执行后用 `cmp` 与永久 fixture 逐字节核对。

| 类 | 原 SHA-256 / 大小 | 补丁 SHA-256 / 大小 | descriptor 改动 | `put` Code 与 opcode 偏移 |
| --- | --- | --- | --- | --- |
| `RawBool` | `e38d0e46a055d5183001d2e51ed329c2a0f121409f27ead257d34f6ab76317e3` / 172 B | `32286d138ace6a328a8c5ba7d0a0d66fadb4bf9dbbab79385e6dc3a01dcd6c2b` / 172 B | `([III)I` → `([ZII)Z` | 长度 8；`2a1b1c4f2a1b2eac`；BCI 3 `iastore 0x4f→bastore 0x54`，BCI 6 `iaload 0x2e→baload 0x33` |
| `Order` | `63dc18e93763ffbf9d6ce73a299a7d140023eb278dd2d430f6e47d208bb55ef0` / 532 B | `31d8fc3394c39cfe9a9e62da435af9245d2f26cd24e62e077e7310e1bf97d414` / 532 B | `([I)[I` → `([Z)[Z`；`([II)I` → `([ZI)Z` | 长度 16；`2ab80014b800181bb8001c4f2a032eac`；BCI 11 `iastore 0x4f→bastore 0x54`，BCI 14 `iaload 0x2e→baload 0x33` |

Descriptor 长度保持不变，Code 长度与其它字节不变。完整 manifest 在 fixture `patches/RawBool.json` 和 `patches/Order.json`。

## 执行结果

使用 `java -Xverify:all` 执行两份冻结补丁 class，stdout 与 `expected/RawBool.stdout`、`expected/Order.stdout` 一致。RawBool 覆盖 0、1、2、3、-1、-2、`Integer.MIN_VALUE`、`Integer.MAX_VALUE` 写入并读取的值；Order 对成功、null 数组、越界、value producer 抛错均观察到 `trace=123`，且异常类型依次为 `NullPointerException`、`ArrayIndexOutOfBoundsException`、`IllegalStateException`。运行器通过反射调用补丁后的真实 descriptor。

## 修前工具阶段（来自冻结审计日志）

修前指本文所引 `openspec/evidence/java-syntax-2026-09-25/boolean-array-lowbit/reproduced/` 审计运行，不是共享工作树当前可能含有的生产源码状态。其 Jarde CLI SHA-256 为 `f99024d22e000d2e28cb1f8d95acc0880bdfea2445343f38b476d04b7d115273`，JADX 为 1.5.6，Javac/OpenJDK 为 23.0.1。

- JADX 对 RawBool、Order 的反编译命令均成功，原样生成的 `int`→`boolean` 赋值源码在 `javac --release 8 -g:none` 阶段失败（exit 1）。记录在审计日志 `jadx-{RawBool,Order}.json`、`javac-jadx-{RawBool,Order}.json`；RawBool 诊断指向 `zArr[i] = i2;`。
- Jarde CLI 对两个 class-source 请求均退出 0，但不是完整恢复：RawBool 在 `bastore` BCI 3 保留 `@bytecode 3`；Order 在 BCI 11 保留 `@bytecode 11 1 4 8`。日志 `jarde-RawBool.json` 与 `jarde-Order.json` 保存完整 stdout/stderr 及报告。
- 以上日志是修前阶段的来源；本次不运行并行修改中的 Jarde 代码来推断修前状态，也不将当前输出记作修前证据。

## 复现命令

以下命令只向私有 `/tmp/jarde-bastore-fixture-output` 写 Java class、patch manifest 与 runner class；`cmp` 检查固定产物。JVM 阶段用 `-Xverify:all`。执行目录可在本机改为另一私有临时目录。

```sh
OUT=/tmp/jarde-bastore-fixture-output
mkdir -p "$OUT/original" "$OUT/patched" "$OUT/runners"
javac --release 8 -g:none -d "$OUT/original" \
  tests/fixtures/p3-boolean-array-stores/RawBool.java \
  tests/fixtures/p3-boolean-array-stores/Order.java
python3 openspec/evidence/java-syntax-2026-09-25/boolean-array-lowbit/patch_classfiles.py RawBool \
  --source "$OUT/original/RawBool.class" --destination "$OUT/patched/RawBool.class" \
  --manifest "$OUT/RawBool.json"
python3 openspec/evidence/java-syntax-2026-09-25/boolean-array-lowbit/patch_classfiles.py Order \
  --source "$OUT/original/Order.class" --destination "$OUT/patched/Order.class" \
  --manifest "$OUT/Order.json"
cmp "$OUT/patched/RawBool.class" tests/fixtures/p3-boolean-array-stores/v8/RawBool.class
cmp "$OUT/patched/Order.class" tests/fixtures/p3-boolean-array-stores/v8/Order.class
javac --release 8 -g:none -cp "$OUT/patched" -d "$OUT/runners" \
  tests/fixtures/p3-boolean-array-stores/RawBoolRunner.java \
  tests/fixtures/p3-boolean-array-stores/OrderRunner.java
java -Xverify:all -cp "$OUT/patched:$OUT/runners" RawBoolRunner
java -Xverify:all -cp "$OUT/patched:$OUT/runners" OrderRunner
```

## 集成测试观察

在共享工作树当前状态运行 `CARGO_TARGET_DIR=/tmp/jarde-bastore-fixture-target cargo test --test p3_boolean_array_stores -- --nocapture`，1 项测试通过。测试严格验证两份固定补丁 class，再分别编译执行完整 Jarde 类并与 frozen stdout 比较。此结果只描述当前工作树；共享生产代码已有并行修改，因此不代表修前 RED 结果。修前拒绝阶段仍以本记录上面的冻结审计日志为证。
