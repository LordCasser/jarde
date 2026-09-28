## 1. 固定语义和负例

- [x] 1.1 以固定 class/probe 的 33 个 BCI、两条异常行与同布局 opcode 为基线，扩展可观察运行到 iterator/hasNext/next/逐项调用抛错，验证原类与固定 JADX 的 Java 8 全源码运行逐路径一致，并记录差异或限制。
- [x] 1.2 制作 verifier 有效的第二循环入口、不同列表/调用目标/循环操作、清理自保护、原 Throwable 改写与额外出口近邻；每个冻结 SHA、`java -Xverify:all` 路径及当前拒绝，验证负例不依赖畸形 class。

## 2. handler 组件的循环事实

- [x] 2.1 在 normal-flow 的现有图中有界证明方法入口不可达组件的唯一根与局部 dominance/自然循环，保留 `dominates` 原语义；固定 Test11 的 BCI 48 成为循环头且 `{48,58}` 不再误判不可约，并以单元测试验证真正多入口、零入口、异常边直入循环及预算/取消仍拒绝。
- [x] 2.2 仅将受证局部循环事实交给区域构造，验证普通方法入口循环、已有 catch-join 与 irreducible 控件的判定保持不变；单独通过预检不算本 change 完成。

## 3. 两份迭代清理的 FINALLY 证明

- [x] 3.1 在现有 FINALLY Guard 内对两条真实 catch-all 行、两份完整迭代副本、同一列表入口值/符号调用/逐项值、单入口回边/退出与原 Throwable 的 SSA 身份建立有界证书；用固定样本命中和 1.2 全部近邻拒绝验收。
- [x] 3.2 在现有 Region/Builder 内一次取得正文及两份清理的所有权，投影一个可重编的 `try/finally` 与一条迭代循环；全部 33 个 BCI 有 source map，失败/预算/取消回滚而不留下半个结构，并用定向 AST/来源测试验收。

## 4. 完整三方验收

- [x] 4.1 用 fresh CLI 将固定原类、固定 JADX、Jarde 完整 Java 8 类分别重编，以 `java -Xverify:all` 对照 1.1 所有路径；原行为与 Jarde 逐字一致且 1.2 有效近邻保持拒绝。
- [x] 4.2 回归普通循环、真不可约、旧两/三/四/五行 finally、Test16/17 完整回放；运行 `cargo test -p jarde-java --tests --locked`、workspace check、fmt、OpenSpec strict、diff check，清理专用 Cargo target 并记录结果。
- [x] 4.3 root 独立核组件入口、双循环 CFG/SSA/异常所有权、三方运行与来源；仅将固定 Test11 Java 8 子形态记为恢复，并更新 CF-16 清单。
