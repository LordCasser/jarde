## 1. 取证与基线

- [x] 1.1 重放固定 L5/D1（SHA 核对）：javap 双面块/边差异（副作用对区域计费与引注落点的影响）；读引注触发条件，定失败闭合判定落点；记录双面基线（L5 静默错编 diff、D1 拒绝）。
- [x] 1.2 构造并冻结至少三个 verifier 有效变体/负例：throw 形态、双 return 形态、引注区含 return 的不可恢复补丁类（闭合判定触发，保持整方法拒绝）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 恢复与失败闭合

- [x] 2.1 do-while(false) 体拥有 return 出边（design 决策 1）；L5/D1 恢复、整类重编运行与基线逐字一致（`early`/`early:5`）。
- [x] 2.2 失败闭合全局判定（决策 2）落地；L5 形行为差测试钉死（不允许可编译且行为不同）；既有 do-while/dj/return 形态与全部既有测试 diff 逐字不变。

## 3. 回归与验收

- [x] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿、fmt、CI 完整 `-A` 清单 clippy、`openspec validate --all --strict`、diff check；磁盘纪律同前。
- [x] 3.2 L5/D1 与变体三方对照：原 class/固定 JADX Java-input/Jarde 重编 `java -Xverify:all` 逐路径一致；记录输出 SHA。
- [x] 3.3 root 独立复核恢复归类、闭合不变量与三方行为，更新 CF 账本与巡查记录。（root 于合并主线 8140c76d 复核：D1（`early:5/d3/f2`）与 L5（含 `early`）整类重编行为逐字一致——**三处静默错编面全消**〔L5 形、D1.retInDoWhile（dj 后主线亦静默化）、D1.retInIf（巡查时未验行为的既有静默面 `end`≠`f2`〕〕；闭合不变量守卫（throw 面整方法拒绝、完成性门槛 JLS 14.22）与守卫测试复核认可；2851/0、fmt/openspec 249/249。**边界翻转裁决接受**：cf07/cf08 两负例有意泛化（叶证明脱离单一值形状，断言可追溯）、retInIf/retInPlainDo 恢复优于 spec 字面"逐字不变"——该场景书写时误以为两形态健康，行为正确性优先。字面 do-while 包装、语句级引注闭合、return 臂 concat 折叠登记后续。）
