# Capture evaluation boundary

这是对“捕获调用结果会被搬到 lambda 每次调用时重新执行”的反证审计。原始输入把
`CaptureSupport.next()` 结果留在操作数栈上直接交给 `invokedynamic`；精确 Code 补丁
删除中间 store/load。为隔离整类 `lambda$` helper 名称冲突，池中的实现方法按同长度
改名为 `bridge$create$0`，方法体与 bootstrap 引用关系保持不变。

patched class 为 834 bytes，SHA-256
`f36f1920e3043f3042bd81e3a3cf9380412f8933e61fb4acca6f1daccf6c33f7`。
`java -Xverify:all` 的 4 组模式共 10 行与 JADX 完整输出一致；jarde 有 1 处引用、
完整 javac 失败。其保留一次独立调用，并在 lambda 捕获的 replayable 检查拒绝该输入。
因此本例没有发布错误的正常 Java，不能登记为已证实的重复执行缺陷。

`binding-hypothesis/` 用非 final 的已赋值局部接住调用结果，再在 lambda 中只读该
局部，完整候选类执行 10 行与原 class 相同。这只是设计候选，未作为 jarde 验收。
未来扩展应复用 `preserve-deferred-value-order` 的成功声明绑定；不能直接删除
`unreplayable` 保护，也不应另建捕获缓存或合并合成 helper 呈现问题。
