## 1. 固定四行与反例

- [x] 1.1 重放固定 Test17 的 `test()I`：核 pinned JADX/test SHA、原与可观察 probe 的逐 BCI/opcode/异常表、原/JADX 完整 Java 8 类八路径；记录修前 Jarde 目标安全拒绝。
- [x] 1.2 冻结 verifier 有效的错误调用目标、异常行范围/顺序、保存返回值或 Throwable 改写、外部清理入口/自保护等近邻；各自至少一条真实运行路径并记录 hash。

## 2. 四行四副本证明

- [x] 2.1 在现有 FINALLY pass 内增加独立有界四行证书，核两具名 handler、两段 catch-all 覆盖、四份同目标 `invokestatic ()V` 清理与 18 个 BCI；不改变 Test16 两行及旧三/五行准入。
- [x] 2.2 证明 CFG 无未声明边、具名 catch 参数和第二 catch 的保存/返回 SSA 身份、共同 `0` 返回、catch-all 原 Throwable 重抛以及清理异常不自捕获；近邻拒绝，预算/取消停止。

## 3. 区域与一次源码投影

- [x] 3.1 在 `Region::Guard` 有界构造一个 try、两个真实 catch 与完整物理所有权；失败回滚访问状态。第一个 catch 空，第二个 catch 只表达提前 `return 1`。
- [x] 3.2 复用 Builder 的 Try/Return/finally AST、来源映射和单 checkpoint，输出一次清理；失败/停止不发布半个 try，18 个 BCI 均可查询。

## 4. 三方验收

- [x] 4.1 fresh CLI 原/JADX/Jarde 完整 Java 8 类重编，八路径 `java -Xverify:all` 逐字一致；1.2 所有有效近邻拒绝。
- [x] 4.2 回归 Test12–16、两/三/五行 finally、具名 catch、TWR/monitor；`cargo test -p jarde-java --tests --locked`、workspace check、fmt、OpenSpec strict、diff check，并清理专用 Cargo target。
- [x] 4.3 root 独立验收四行/四副本 CFG/SSA、八路径和 18 个 BCI 来源，更新 CF-16 清单；仅标记固定 Test17 Java 8 子形态。
