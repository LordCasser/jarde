## 1. 冻结边界

- [x] 1.1 固定 `tests/p3_sync_return.rs` 的 `Locked.locked` Java 8 源、class、runner 与哈希；记录当前 `saved0` 输出和原 class/JADX/Jarde 的重编执行结果，确认失败是直接表达式质量而非错值。验证：`javac --release 8 -g:none` 重编 class 与冻结字节相同，`java -Xverify:all` 结果逐行可复核。
- [x] 1.2 构造 verifier-valid 的独立效果反例：返回值产生后、正常 `monitorexit` 前执行有计数或抛错的调用；再加一个 Guard 不能证明的退出反例。验证原 class 在正常/异常输入的次数、旧值和异常优先级，保存完整字节码与 hash；当前呈现若拒绝须保留相关 BCI，不能拿不可执行 class 作为拒绝依据。

## 2. 收紧值放置证明

- [x] 2.1 从既有已证明的同步 Guard 向 Builder 的绑定准备交接唯一正常退出 BCI、对应返回及 body 范围；不按指令邻近或 `facts()` 中任一退出猜测。验证：正例只有该退出被认领，异常处理器退出、嵌套/未证明 Guard 均不获得该身份，预算/取消中途无部分计划。root 通过 `Result` 原子返回与 Builder 创建顺序审读了停止边界，见[独立验收](../../evidence/java-syntax-2026-09-24/guarded-return-expression/root-review.md)。
- [x] 2.2 只对最终 consumer 属于同一 Guard 返回、生产者在其 body 内的值，在独立边界判断中略过该正常退出；其它效果与不完整依赖仍保存或拒绝。验证：`p3_sync_return` 恢复 `return this.n`、无 `saved0`；1.2 的调用计数/异常顺序不变，`preserve-deferred-value-order` 的直线错序与 guard 控制回归不退化。

## 3. 独立验收

- [x] 3.1 对 1.1 的同一冻结 class 用 JADX 1.5.6 与当前 Jarde 恢复完整类，执行 Java 8 重编、`java -Xverify:all` 的值对照；对 1.2 的合法反例执行原 class/JADX 的次数与异常对照，并核查 Jarde 的明确拒绝与真实 BCI。核查正例来源锚点包含真实字段生产与 Guard 退出，essential/all 正文一致。结果见[独立验收](../../evidence/java-syntax-2026-09-24/guarded-return-expression/root-review.md)。
- [x] 3.2 root 复核 2.1 的所有权边界与 2.2 的效果判定，运行定向 Rust、fmt、OpenSpec strict 和相邻 TWR/synchronized 回归；只在所有门槛通过后勾选，不把其它异常局部作用域债务并入本变更。
