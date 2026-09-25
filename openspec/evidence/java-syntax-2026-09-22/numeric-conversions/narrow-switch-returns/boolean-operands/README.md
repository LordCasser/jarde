# Boolean-presented operand at a B/C/S `ireturn`

这是父目录 boolean 控制分支观察的独立边界证据。`BooleanOperands.java` 的唯一业务
方法是：

```java
static boolean run(boolean value) { return value; }
```

它先以 `javac --release 8 -g:none` 编译，再只修改 `run` 的 descriptor：`(Z)Z` 分别
变为 `(Z)B`、`(Z)C`、`(Z)S`。参数 descriptor 始终是 `Z`，方法 `Code` 不变，操作数栈
上的值真实来自 `iload_0` 的 boolean-presented 参数；这不是 int 常量控制分支，也没有
手写 producer。

## 重放与字节事实

在仓库根目录执行：

```sh
python3 openspec/evidence/java-syntax-2026-09-22/numeric-conversions/narrow-switch-returns/boolean-operands/run_audit.py
```

源码 class 为 171 bytes，SHA-256 为
`0857c114b45889f769deb8a9e5b2dea588437cabd6e2c24f66e20b037ae063e5`。descriptor 字符串
位于 class offset 145；每个补丁 class 为 178 bytes，只增加一个 UTF8 descriptor，且
两个 `Code` attributes（构造器和 `run`）的合并 SHA 始终为
`a2ce6f368bda9cc530834807f41f29a22e6366e90ff728f4040cf72a798e5813`，补丁前后相同。
补丁 class SHA 为：B
`e6334168e41f8eb5faf6c7599cb6452ff17986dc744111e804459c548cede81f`，C
`78af87dfe136343ab1cbc2ff4bfe51e02f5987b2cd8c51672799a3e265e2dfb1`，S
`3335d78a1fe7e0f14c3c4ef14fc9172eeec8df2bc42fff610991f03dc96d2aad`。

## 实测结果

原始 Z class 和 JADX 完整输出、jarde 完整输出都通过 `javac --release 8` 和
`java -Xverify:all`，runner 的两个输入输出为 0/1（源 boolean 在 runner 中统一盒装为
整数观察）。三组 B/C/S descriptor class 也都通过 `java -Xverify:all`，每组输出均为：

```text
run(false)=0
run(true)=1
```

这证明 JVM 接受保留 Z 参数、B/C/S 返回 descriptor 与同一 `ireturn` Code 的组合，并
且窄化结果确实消费了 boolean-presented 值。当前 jarde 对每个补丁只报告一个真实
`@bytecode` quote，但没有把该值发布成返回表达式，所以三组生成文本都因“缺少返回语句”
而无法编译；JADX 三组完整文本均编译并运行成功。该失败是当前恢复边界，不把
JADX 的成功文本当成 jarde 已恢复。

完整三方状态、descriptor patch 记录、Code SHA、CLI SHA 前后校验在
[`summary.json`](summary.json)；所有阶段的完整文本和日志分别位于 `z-original/`、
`b-descriptor/`、`c-descriptor/`、`s-descriptor/`。固定 CLI
`/tmp/jarde-cli-deferred-final-ecab` 前后 SHA 均为
`ecab8244d1709765fa0f2b0effcebe49b9f066eea911843d11d34abec5837330`。
