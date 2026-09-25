# switch join 中 arm 绑定尚未提交的可执行反例

本目录保存一个 source-only、可重放的 scope 反例。`run_audit.py` 先用
`javac --release 8 -g:none` 编译 `SwitchDeferred.run(int)`，再只替换该方法的整个
`Code` 属性；没有手改恢复源码，也没有改生产或运行 Cargo。

## 精确输入与 JVM 验证

patched `run(I)I` 的 Code 为 36 bytes，`max_stack=1`、`max_locals=1`，class major 从
52 降为 49，并移除该 Code 内的 `StackMapTable`。`java -Xverify:all` 成功执行原 patched
class。Code 的关键字节序列和偏移如下：

```text
0  iload_0
1  lookupswitch (2 padding, default +28 -> 29, key 1 +19 -> 20)
20 invokestatic SwitchSupport.value()I
23 invokestatic SwitchSupport.mark()V
26 goto +9 -> 35
29 invokestatic SwitchSupport.other()I
32 invokestatic SwitchSupport.mark()V
35 ireturn
```

原编译 class SHA-256 为
`301f21aa6ac58558ab330d4fd91726bdd4550dff09a3b2ef6efc2871087ffb39`，patched class SHA-256
为 `e684f35d278b59e7b50fdf5919e340749c49b6412d155f4d80c9df4fc6bd4ec5`。完整的
`javap -v -c`、原始/替换 Code hex、属性删除和偏移均在 `summary.json` 与
`patched-javap.txt`。

## 三方结果

当前 CLI SHA-256 前后均为
`feed5c377a439490efb12a2a46bdfa0e94c32dffd3a83bd1583b70fe89317350`。原 class、完整 JADX
正文和各自 helper/runner 都通过 `javac --release 8` 与 `java -Xverify:all`；JADX 6 行
输出逐行等于原 class。

当前 jarde 输出也通过完整编译，但不是等价执行：它在每个 arm 中先写了
`SwitchSupport.value(); SwitchSupport.mark();`，随后在 join `return` 又重新写
`SwitchSupport.value()`；default arm 同样重复 `other()`。因此输出为：

```text
原 class:  1:0:value=101:trace=12
           7:0:value=303:trace=32
jarde:     1:0:value=101:trace=121
           7:0:value=303:trace=323
```

`mode=1` 的 producer failure 与 `mode=2` 的 mark failure 六行均保持异常身份和原有
trace，说明该反例具体暴露的是正常路径上的重复消费；两种 tag、三种 mode 共 6 行，
jarde 与原 class 只有上述 2 行不同。jarde 完整源码和三方逐行输出分别保存在
`jarde.java.txt`、`original.txt`、`jarde.txt`、`jadx.java.txt`、`jadx.txt`，汇总状态见
`summary.json`。

## 代码原因

`build.rs:1361-1365` 明确在构造 switch arms 之前调用 `switch_join`，而
`build.rs:1367-1373` 才进入每个 arm 的 `self.arm(...)`。join 返回值的 `render_value`
因此发生在 arm 内的 `bind_value` 提交之前。当前 patched bytecode 的真实 value producer
在 arm 内、mark 之后才有独立消费边界；预渲染 join 时看不到成功 `Binding`，沿旧 inline
路径把 producer 写进 join return。随后构建 arm 又写出 producer 与 mark，形成一次重复
执行和 `trace=121/323`。

这不是非法栈或普通 switch 控制流问题：JVM verifier、JADX 和原 class 均通过。修复需要
让 switch join 的返回呈现使用同一次已证明的 arm binding，或在绑定尚未可提交时保留
join/arm 的完整来源拒绝；不能在 join 预渲染阶段回到 producer 处 inline。
