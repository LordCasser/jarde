# CF-14 主线独立复核

root 以保存的主线 CLI `/tmp/jarde-root-cli-cf08`，从归档三个原 class 独立生成完整 Jarde 源码；普通、嵌套、独立 hash 样本的 SHA-256 分别为 `2359246428d71833a0b8fe98ba711e36c826f26779691e6d340f96b6ca6c8832`、`76beb21fc2c13ada550d16a5b64640b773184efc9ed26f1d5dfe862c81d85e6b`、`dcc3d93b5254e60165b2bbc0623d38ff0a09bcbf4ceec9655057bb70ca5628ba`，均与审计归档逐字节相同。

root 分别重新以 Java 8 编译原始、固定 JADX、Jarde 的完整类源码与同一 runner，并用 `java -Xverify:all` 运行可编译产物。普通碰撞/分组 String switch 三方 8 行逐行一致；嵌套 String switch 原/JADX 5 行一致，Jarde 因 `choose(String)` 缺返回而编译失败；独立 hash 用途原/Jarde 6 行一致，固定 JADX 引用未定义 `r0` 无法编译。Jarde 的独立 hash 用途没有被误折成丢失副作用的 String switch。审计报告中误写的 `NestedStringSwitchAudit.run(String)` 已在主线改为实际方法名 `choose(String)`。

CF-14 仍是明确恢复差距；下一个实现任务应先补齐嵌套分派的 Region owner，再核对 slot 4 的声明范围。当前证据不支持把失联 block 武断归因于 normal-flow 某一种被过滤的边。
