## 1. 取证与基线

- [ ] 1.1 重放固定 BR 家族五类（SHA 核对）：读两门判据上下文（`src/facade.rs:29499` 擦除返回门、`bridge.rs` 体形门）与既有投影通道；确认门 1 换层级 walk 对"继承需求"判定输入的影响、门 2 参数 cast 形的可重建性；记录三类基线（两门各自拒绝文本 + javac name-clash 报错）。
- [ ] 1.2 冻结至少四个变体/负例：源返回与桥返回无层级关系（拒绝）、cast 目标≠源级参数类型（拒绝）、效果不纯桥（既有拒绝）、多层级协变（接口→父接口）；各自 `java -Xverify:all` 通过并记录实现前后行为。

## 2. 两门扩展

- [ ] 2.1 门 1 换快照层级 walk（design 决策 1）；`BR$Base` 桥被投影隐藏、真实协变覆写保留。
- [ ] 2.2 门 2 接受规范参数 cast 形（决策 2）；`BR$StrBox`/`BR$Impl` 桥被隐藏；BR 家族三类整类 `javac --release 8` 通过、`java -Xverify:all` 运行与 orig.out 逐字一致（`Base`/`s`/`0`）。
- [ ] 2.3 负例保持拒绝；既有桥投影正例与 `negative/`、`orphan/` 负例 diff 逐字不变；预算/取消原子性不变。

## 3. 回归与验收

- [ ] 3.1 `cargo test --workspace --tests --locked --no-fail-fast` 全绿（含既有 bridge 单元/模式测试、snapshot-hierarchy-widening、access$/lambda 消隐家族）、fmt、CI 完整 `-A` clippy、`openspec validate --all --strict`、diff check；磁盘纪律：每轮构建前 `df -h /`，低于 12Gi 先 `cargo clean`，报告前必 clean。
- [ ] 3.2 BR 家族与变体三方对照：原 class/固定 JADX（dev，其改名路线仅作对照不作准入）/Jarde 重编 `java -Xverify:all` 逐路径一致；含经接口引用的调用路径；记录输出 SHA。
- [ ] 3.3 root 独立复核两门判据、可重建性论证与三方行为，更新账本与巡查记录；`project-proved-bridge-forwards` 自身未勾项按独立债务另行登记，不在本片代办。
