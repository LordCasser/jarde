# failed declaration：shift 被拒绝时的保存边界

这是一个 source-only、可重放的失败声明边界。`run_audit.py` 先用普通
`javac --release 8` 编译：

```java
static int run(int n) {
    int local = n << 1;
    return Support.take(local);
}
```

随后只在 `take` 的返回值与 `ireturn` 之间插入已有常量池中的
`invokestatic Support.mark()V`。patched `run(I)I` 的实际指令为：

```text
0 iload_0
1 iconst_1
2 ishl
3 istore_1
4 iload_1
5 invokestatic Support.take(I)I
8 invokestatic Support.mark()V
11 ireturn
```

原 class 的 SHA-256 是
`1e979a725f0f17d0686f1678537d173b725c728aeb1504b6c2aef8092cf21070`，patched class 的
SHA-256 是 `cc8901164d3aac1206b65fc03c69d8798359b88c0bd300c5202ef614ccc79d0b`。Code
长度从 9 变为 12，完整的替换 hex、`javap -v -c` 和编译日志在本目录。patched class
通过 `java -Xverify:all`。

## 三方执行

输入 `5`、`-3` 与 producer 正常、producer 抛错、marker 抛错三种 mode 共 6 项。原 class
与完整 JADX 恢复均通过 Java8 编译及 `-Xverify:all`，逐行一致：

```text
5:0:value=110:trace=12
5:1:error=java.lang.IllegalStateException:same=true:trace=1
5:2:error=java.lang.IllegalStateException:same=true:trace=12
-3:0:value=94:trace=12
-3:1:error=java.lang.IllegalStateException:same=true:trace=1
-3:2:error=java.lang.IllegalStateException:same=true:trace=12
```

审计时当前 CLI SHA-256 为
`ecab8244d1709765fa0f2b0effcebe49b9f066eea911843d11d34abec5837330`，前后未变化；固定
首版 `/tmp/jarde-cli-deferred-first-feed5c` 为 `feed5c377a439490efb12a2a46bdfa0e94c32dffd3a83bd1583b70fe89317350`，两者输出正文相同。

当前 jarde 正文保留了真实 BCI 来源，但没有发布非法保存局部或重算 `take`：

- BCI 2 的 `ishl`、BCI 3 的 `istore` 分别被引用为不可恢复；
- BCI 5 的 `take` 通过 BCI 4 的 `iload` 作为 derived source 被引用；
- BCI 8 的 `mark` 作为真实 direct source 保留；
- BCI 11 的 `ireturn` 以 BCI 5 为 derived source 保留。

这些关系也写入 `jarde-full.json` 的 `run(I)I` source-map segments。run 正文的可执行
部分只有 `Support.mark();`，没有 `Support.take(...)`、`savedN` 或 `localN`。因此完整
jarde 类按预期无法编译（缺少 `return`），而不是发布一个引用未声明局部或在 marker 后
重算 `take` 的正常 Java 方法；`jarde-javac.log` 保存了该真实失败。

## 结论

shift 是当前范围外的真实 producer failure。保存 `take` 的候选在声明初始化时读取
`local1`，但 `ishl` 无现有可呈现表达式，原 local 的 Declare 不能提交；后续
`bind_value` 将声明降为 fallback，并通过 `binding_refused` 使最终 return 保留来源而不
重算调用。该输入验证了失败 Declare、失败局部传播、producer/consumer/mark/return 来源
和“无未声明 saved 名发布”的边界，不要求本例恢复为可编译正文。
