# recover-unary-negation 验证记录

## 输入与工具

fixture 是自写的 `tests/fixtures/p3-unary-negation/UnaryNegation.java`，没有使用外部样本。编译环境为 `javac 23.0.1`，命令为：

```text
javac --release 8 -g:none -d tests/fixtures/p3-unary-negation/v8 tests/fixtures/p3-unary-negation/UnaryNegation.java
```

提交 class 大小为 1194 bytes，SHA-256 为 `9520067e939f16e893953ca89cf0fa5bd9543af9da6125dbded9c91e3bae368d`。jadx 版本为 `1.5.6`。原始源码和 class 保存在 fixture 目录，jarde 的可重放断言在 `tests/p3_unary_negation.rs`。

## 修前失败

在加入 `Negate` 前运行：

```text
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test --locked --test p3_unary_negation four_direct_numeric_negations_are_recovered -- --nocapture
```

`negInt` 的结果为 `Mixed`，正文只有 `// @bytecode 1`，并报告 `ireturn` 读取 `Other` 生产者。这证明四个直接取负在修前是缺口，而不是测试仅检查最终文本的空断言。

## 三方输出摘录

jadx 1.5.6 对同一 class 的代表性输出：

```java
public static int negInt(int i) {
    return -i;
}

public static int negatedCall(int i) {
    return -consume(i);
}
```

jarde CLI（`--policy single-class --release 8`）对同一 class 的代表性正文：

```java
return -arg0;
return -(-arg0);
return -(arg0 + arg1);
return arg0 * -arg1;
return arg0 / -arg1;
int local1 = -arg0;
return consume(-arg0);
return fail(-arg0);
return -consume(arg0);
return -fail(arg0);
```

`unsupportedOperand` 保留了 `consume(7);` 一次，并把无法呈现的 `i2b` 和最终 return 分别保留为 `// @bytecode 5`、`// @bytecode 7`；取负不会吞掉调用生产者。

## 受控执行

ignored JDK 对照测试命令：

```text
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test --locked --test p3_unary_negation jdk_execution_matches_the_original_for_numeric_boundaries_and_effects -- --ignored --nocapture
```

结果为 `1 passed`。对照只覆盖该自写 fixture：整数最小值、long 最小值、float/double 正负零与无穷的 raw bits、NaN 分类、byte/char/short 提升、嵌套和乘除分组、一次调用计数以及两个异常路径。没有把 NaN payload 或该样例之外的语义推广为契约。

## Rust 验证

```text
CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test --locked -p jarde-java --lib
93 passed

CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 RUST_TEST_THREADS=1 cargo test --locked --test p3_unary_negation -- --nocapture
7 passed, 1 ignored

CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo build --locked -p jarde-cli
finished successfully
```

fixture census 和 `corpus-fingerprint.json` 随同本轮后续 special/cast fixture 集中更新：取负+special 第一阶段为 `(85,417,74,197,8)`，cast 最终冻结后为 `(86,431,74,197,8)`，reader census 复跑通过。指纹正常校验为 5 passed / 1 ignored。相对更新前快照新增 45 个条目，其中取负 2、special 7、cast 2，其余 34 是本轮前已有但旧清单未列的语料，不能算作本 change 新增。日志分别为 `/tmp/jarde-reader-census-final.log`、`/tmp/jarde-corpus-normal-final.log`、`/tmp/jarde-corpus-diff-final.log`。

## 主代理独立验收

主代理随后独立复跑了相邻回归和整类语义审计，日志保存在：

- `/tmp/jarde-neg-root-regression.log`：43 个相关测试通过，1 个 ignored。
- `/tmp/jarde-neg-root-jdk.log`：ignored JDK 对照 1 个通过。
- `/tmp/jarde-neg-root-lib.log`：`jarde-java` library 93 个测试通过。
- `../../evidence/java-syntax-2026-09-22/negation`：整类 8 方法的 Java syntax 验收；20 个字节码引用降为 0，原 class 与恢复文本执行 trace 一致。

严格 clippy 仍有既存的 `region.rs::try_level` `type_complexity`，未通过新增 `allow` 隐藏，也未混入本 change。fixture 当前 class 仍为 1194 bytes，SHA-256 仍为 `9520067e939f16e893953ca89cf0fa5bd9543af9da6125dbded9c91e3bae368d`。
