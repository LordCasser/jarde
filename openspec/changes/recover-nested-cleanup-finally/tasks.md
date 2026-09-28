## 1. 固定事实和反例

- [x] 1.1 复核固定 JADX Test4 源/class 哈希、Java 8 同形字节码、四行异常表与当前两级拒绝；保存独立 CLI 的原/JADX/Jarde 完整源码及正常运行。
- [x] 1.2 为至少错清理调用、错接收者/效果和错异常覆盖制作 verifier 有效的近邻，观测可区分行为；control 加上正文异常与清理异常的组合路径，明确它不是固定目标 class，且 Jarde 对其三行形态可继续拒绝。

## 2. 联合证明

- [x] 2.1 在既有 FINALLY 四行分派内先尝试 Test4 专用证书，不放宽 `TwoCatchReturnFinally`；精确核范围、catch 类型、self row、所有异常和正常边。
- [x] 2.2 以 SSA 和符号调用事实核正文、两份 `close(); delete(); pop` 的接收者/顺序、空 catch 参数、原异常身份及正常/异常完成；为所有 BCI 计费、轮询和建立所有权。

## 3. 结构输出

- [x] 3.1 复用现有 Guard/Region/Try AST 构建一份外层 `try/finally` 和 finally 内层 `try/catch(IOException)`；空 catch、声明绑定与两份物理清理只写一次。
- [x] 3.2 核 source-map 全 BCI、预算/取消原子回退、无法绑定局部或未获证调用的安全拒绝；旧 finally 证书不回归。

## 4. 三方验收

- [x] 4.1 fresh CLI 对固定目标的原/JADX/Jarde 完整源码做 Java 8 重编、`-Xverify:all` 正常路径；对独立 control 的异常路径做原/JADX 事件/终局异常比较，记录 Jarde 的安全拒绝，不混淆两者证据。
- [x] 4.2 verifier 有效近邻全部拒绝；运行定向 FINALLY、Region、class-source 回归，workspace check、fmt、OpenSpec strict 和 diff check；清理专用 Cargo target。
- [x] 4.3 root 独立验收固定物理形状、三方重放、反例和来源，更新 CF-16 清单；只标记 Test4 子形态，不宣称所有 finally 完成。
