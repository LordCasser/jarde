## 1. 固定目标与可验证近邻

- [x] 1.1 从固定 Test16 提取不变的 `test()V`，制作仅辅助方法可观察的 Java 8 完整类；`replay.sh` 核两份目标方法逐 BCI/opcode/异常行相同、原/JADX 六路径 `-Xverify:all` 一致，并记录当前 Jarde 安全拒绝。
- [x] 1.2 制作 verifier 有效的调用目标不同、异常行范围/顺序变化、原 Throwable 改写、清理外部入口等近邻；每个 class 用 `java -Xverify:all` 及至少一条可观察路径确认其真实行为，固定 hash 和异常行。

## 2. 两行三副本证明

- [x] 2.1 在既有 FINALLY Guard pass 内证明两条同范围异常行、空具名 catch、三份解析目标相同的静态零参 void 调用与唯一正常返回；定向测试核 11 个物理 BCI、行 ordinal、调用 target 和所有者。
- [x] 2.2 证明每份清理在保护范围外、Canonical CFG 无额外入口/出口、具名参数不产生副作用、catch-all 保存并重抛原 Throwable；1.2 的近邻不得获证，预算/取消测试必须返回 Stop。

## 3. 有界结构和唯一源码

- [x] 3.1 在现有 Region::Guard 路径构造受保护正文与真实空 `catch (Exception e)`，所有 11 个 BCI 被唯一拥有；区域失败必须恢复访问状态，定向测试核两行而非虚构第三行。
- [x] 3.2 Builder 在同一 checkpoint 中输出唯一 `try/catch/finally`，三份调用合成一次源码调用且保留各自来源，具名 catch 无多余语句，失败或停止不发布半成品；检查目标源码完整、来源可查和预算/取消原子性。

## 4. 三方验收与回归

- [x] 4.1 用 fresh CLI 重编原/JADX/Jarde 三份完整 Java 8 类，六路径 `java -Xverify:all` 逐字比较；1.2 所有有效近邻均安全拒绝，不能靠受控补丁通过 Jarde 路径。
- [x] 4.2 运行固定 Test12–14、两/三/五行共享 finally、具名 catch、TWR/monitor 回归及 `cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、fmt、OpenSpec strict、diff check；清理专用 Cargo target。
- [x] 4.3 root 独立核对真实异常行/三副本 CFG/SSA、源图与六路径重编运行，写验收记录并更新 CF-16 清单；仅标记固定 Test16 Java 8 切片。
