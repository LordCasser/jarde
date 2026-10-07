# Root 独立验收（2026-10-07，合并）

## 判据逐项

1. **落点复核**：`results/01-locate-ordering-logic.md` —— 次序决策在 `ctor_order.rs::present_prologue_first`（单调用点 `build.rs:8082`），member_inner/facade 属分配位投影/伴生实例化（`DB$2` 超类不可拼→匿名投影拒绝=池拼写域，非本片）。门控：仅加接受臂，`DB$2` 翻转、`DB$1`/宿主逐字节。
2. **第三条件裁定（本次验收的核心）**：design 决策 1 的两条件**不足**（会重排 `AnonymousSuperDispatch$1` 并破坏守卫）——实现者按守卫同源的声明视图事实（`Inputs::class_methods`，`class_fields` 的姊妹）加第三条件"调用不可观察该组字段"（`Object.<init>` 臂逐字保留 + "只声明构造器与 clinit"新臂），并walk 每个实参的 SSA 闭包（数据流非拼写：被移动字段读/任何调用即拒）。**追认为正确强化方向**（收紧而非放宽）；四个顺序敏感 fixture（声明 reader 方法）全部保字节序——`ctor_reorder_dispatch_guard` 2/2、`fixture_behavior_guards` 7+6 root 复跑绿。
3. **root 亲测**：DB 家族三单元（池名改写 `DB1`/`DB2`）联编 `javac --release 8` exit 0、运行 `2/z` 与原类一致——**捕获形 ctor 重排后整类行为正确**；伴生渲染 `super(); this.val$s = arg1; this.add(...)` 原序恢复。
4. **可判伪性**：实现片自检——关新臂 4 测试中 2 失败；去掉 walk 调用臂探针测试失败（均回退）。
5. **门禁（权威口径）**：全量 exit 0、**333 targets ok、0 FAILED**；fmt OK；CI 逐字 clippy `Finished` 0；openspec **307/307**；oracle ignored 3/3。
6. **corpus**：fixtures 857 类→2 delta（新 DB$2 双腿）、evidence 2725→1（巡查 db.jar 的 DB$2）；`CallArg$1` 仅 `analysis_steps` +1（walk 计费，行为不变）；其余逐字节。census 844/3702→857/3723（其自身再测指令），指纹 +17 纯增。
7. **CI**：合并推送后 run 为准（监控在案）。

## 残余裁定

- **TH$1（Thread 匿名）保持逐字**——声明 `run()` 即有可观察面，巡查"trailing initialization"判别词不是代码判据；证 `Thread.<init>` 无害需无持有的 body，登记边界 ✓；
- `super(compute())` 形保守保序（非 javac 形）；`ReadArg$1` 不可分析——按当前事实钉住带恢复日更新标签 ✓；
- 两处 stale-pin 更新（p3_patterns 生成器 pin 双侧断言、reader 人口再测）追认 ✓。
