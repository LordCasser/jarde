# CF-16 拼接保存返回：root 独立验收

固定 `FinallyOnce.class` SHA-256 为 `3ad6857285368c95c3176a520300c4abba09084443fe7fc08e8972330e9cedd2`；同布局最小完整类 SHA-256 为 `49ae3d9438d923ac047110a268424cc0e855920a9a82976d48cf99ddf4d581a6`。`handled(boolean)` 的 BCI/opcode 序列和三行异常表与原 class 逐项相同：`[4,21)→31 IllegalArgumentException`、`[4,21)→65 any`、`[31,55)→65 any`。固定 JADX HEAD 是 `2fb1b16386941660fda07e9017285aec40fcb37f`。

主线审阅实施提交 `be04fa173fd746b1efc0affffdaacb3796443566`：Guard 沿用同轮只读 `concat::Plan`，仅当链的全部指令占有 BCI 32–51、位于 catch-all 保护范围、尾值 BCI 51 唯一送到 store54 时，现有 shared-finally 证书才接纳它作为先求值的保存返回。三份清理仍由原有副本/异常行和 SSA 值身份共同证明；Region 和 Builder 没有新增第二套 finally 生成路径。非拼接调用、额外栈消费者、缩短的异常范围及预算/取消近邻保持拒绝。未发现该切片需要新增机制。

root 从该提交新建 `/private/tmp/jarde-cf16-concat-root-target` 构建 CLI，运行 `bash tests/fixtures/p3-concat-saved-finally/replay.sh /private/tmp/jarde-cf16-concat-root-target/debug/jarde-cli /private/tmp/jarde-cf16-concat-root-replay`。同布局完整类的原 class/Jarde Java 8 重编与 `java -Xverify:all` 均为 `normal:1`、`caught:arg:1`；固定 JADX 为 `normal:2`、`caught:arg:1`，其正常分支多执行一次清理。受控 `getMessage()` 抛错的原 class/Jarde 都输出 `true:java.lang.IllegalStateException:message:1`，其中 `true` 是同一异常对象身份。三个 verifier 有效错误近邻均拒绝 finally 投影。root 的 `cargo test -p jarde-java --test p3_shared_join_finally --locked` 为 14/14；OpenSpec strict、diff check 通过。实施代理另完成 jarde-java 全测试、workspace check 和格式检查。

固定原始完整类的 `main()` 仍有独立引用恢复缺口；本次只验收 `handled` 的同布局最小完整类，CF-16 其余 Test14 条件清理等形态继续单列。
