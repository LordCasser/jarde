## 1. 冻结事实与验收载体

- [x] 1.1 冻结 `FinallyOnce.handled` 原 class SHA、三行异常表、保存返回与三份清理 BCI，重放原 class/JADX/Jarde 完整 Java 8 基线；记录原 `normal:1` 与 JADX `normal:2`，不可把编译成功当语义通过。
- [x] 1.2 制作只调整无关方法的同 BCI/opcode/异常行完整验收类，逐指令比对 `handled`，原/JADX 可重编并 `java -Xverify:all`，覆盖正常、具名 catch 和异常路径。

## 2. 复用拼接证明恢复共享 finally

- [x] 2.1 将同一轮只读 `concat::Plan` 传至共享候选证书，不重复解析拼接；定向测试核 BCI 51 的受证链可见，非拼接 `Invoke` 仍拒绝。
- [x] 2.2 在现有保存返回生产者证明中接受完整处于具名 catch 保护范围、尾值唯一流向 store 的受证拼接；以链缺失、跨界、额外消费者、异常行改动的 verifier 有效近邻核拒绝。
- [x] 2.3 复用现有 Region/Builder 输出两个原值返回、一个具名 catch 和一份 finally；验证完整目标方法无引用、三份清理各有来源、拼接只在 catch 返回位置求值一次，预算/取消失败原子停止。

## 3. 三方运行与主线验收

- [x] 3.1 使用 fresh CLI 将原/JADX/Jarde 完整 Java 8 类分别重编并 `java -Xverify:all`；Jarde 的正常、具名 catch、异常对象/消息与计数等于原 class，单列 JADX 二次清理错误，所有有效错误近邻保守拒绝。
- [x] 3.2 运行共享调用型/字段型/汇合点、Test12/Test13、普通具名 catch、拼接回归，`cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、`cargo fmt --all -- --check`、OpenSpec strict 与 `git diff --check`；清理专用 Cargo target。
- [ ] 3.3 root 独立审阅拼接计划来源、异常覆盖、值身份、区域/Builder 所有权和三方运行，写主线验收记录并更新 CF-16 清单；仅标记该固定切片，不宣称整个 CF-16 追平。
