## 1. Baseline and narrow correction

- [x] 1.1 root核实际native command4失败、源pins和exact clean；对照gate/Registry.enter/Entered::drop及worker/serial共用入口，确认是区间错位而非max竞争，完整失败收据另行保留。
- [x] 1.2 Luna在私有目录按design移动唯一_entered guard至入口，不改阈值、gate算法或调度；root审最小patch、panic/早返回锁与drop边界，定向bulk生命周期/取消/交付/容器持有门及真实重叠复跑通过后验收。

## 2. Integration and delivery

- [x] 2.1 root完成fmt/CI Clippy/MSRV及相邻/P5门，构建最终CLI并以20同argv API核除usage外全部JSON/完整Java/raw maps与已有最终对照精确相同，显式复用其JVM矩阵；source/CLI守卫及clean有收据。
- [ ] 2.2 root两fresh native完整通过并独立审计目标库存、全部具名结果/逐target计数及本片bulk/typed关键门；最终代码单独提交推送并接受自身CI，strict/失败历史/账本/handoff/干净main与target absent闭合，不把局部成功当全局71/612完成。
