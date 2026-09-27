# CF-18 分段 catch/循环固定形态独立验收

root 在合入 `39345a5d`、`d40bb706`、`b8df1556` 后从主分支重新构建 CLI，并独立执行 `replay.py`。两份冻结 class 的原/JADX/Jarde 完整 Java 8 源码均可编译。`HandlerLoopProbe` 三方经 `java -Xverify:all` 均输出 `4:110`；`ExceptionRegionsAudit` 原/Jarde 均输出 `124:115`，固定 JADX 在 `work(2)` 外抛 `IllegalStateException`。Jarde 两份完整源码都没有 `@bytecode`，SHA-256 分别为 `ef8894fdc7c1a95486e1ec1053eab845e39859c2ce811e7f28397736893b9488`、`985e25b5dffa49f00d41cf6012cf08b8efc502df0c54b12519e91e0b121cf7c2`，与实现方归档一致。

root 将同轮 SSA 记录的物理指令 BCI 与重新生成的 `report.json` source-map 主来源和派生来源作集合比较：缩小类 31/31、完整类 47/47，均无遗漏或额外 BCI。内层正常出口 `goto`（16/35）只归入受证 `Try`，回环 `goto`（54/88）只归入对应 `Loop`。证书逐一约束异常表行顺序、handler 的异常独占入口、普通续接、唯一循环更新点和索引 SSA 身份；Region 认领仍需通过已有唯一 owner 检查。八份 `java -Xverify:all` 可加载的近邻负例在独立 CLI 运行中均整方法回退，没有部分 catch。

主分支 `cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、`cargo fmt --all -- --check`、`openspec validate recover-fragmented-loop-catches --strict`、`git diff --check` 全通过。验收仅覆盖上述固定跨区域形态；CF-18 其余测试、异常区组合与循环变化仍待扩验。
