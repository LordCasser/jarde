# Root 独立验收（2026-10-07，合并）

## 判据逐项

1. **门控复核**：`results/01-gating.md` —— 拒绝点=`verify` 消费侧 `written.is_empty()`（读者=类别 1 `pop`，不在 `renders_its_reads`）；门控探针翻转 B5/B6 而 `VoidBetween` 保持 `jre_new_interleaved_effect`@BCI4。**实现者首稿重写 `Sites::owns` 时掉了 bound-receiver 尾巴**（OP/M1 回退）被其 sweep 抓住并修复（双锚逐字节复原）——自查质量高的证据。
2. **diff 审查**：`Site` 声明丢弃 `pop` 为唯一读者（紧随构造调用、读其写出值、单用）∧ 实参全为常量/入口槽 Load/同证明跨步构造——`init.rs` 消费侧 + `build.rs` 语句落点；CST 顺序以两测试钉死（冻结 `VoidBetween` + 组装孪生 `SPC` 均 `jre_new_interleaved_effect` 先于读者检查）。
3. **root 亲测**：B5 全类 0 引注，剥离编译 exit 0、运行输出与 `orig.out` **逐字一致**（ctor println 副作用恢复——本片目的达成）；定向 8+1 ignored 回放绿；`class_source` 102/102；oracle ignored 3/3。
4. **门禁（权威口径）**：全量 exit 0、**331 targets ok、0 FAILED**；fmt OK（合并态复跑通过——上次教训已吸收）；CI 逐字 clippy `Finished` 0；openspec **307/307**。
5. **corpus**：moved=14 全分类（8 恢复且编译运行一致；无更拒/更多引注类）；census `(835,3649,…)→(844,3702,290,2273,8)`，指纹 +17 纯增。
6. **CI**：合并推送后 run 为准（监控在案）。

## root 裁定

1. **D3 池形拼写 delta——追认**：成员类语句位构造在成员投影未证时以通道自身表达式拼写（`new D3$In(new D3())`）呈现——该拼写是**基线既有行为**（消费位手构对照在父提交二进制上同拼写），不可编译=响亮安全形；成员形折叠是池拼写债域的另片路线，本片不越界。
2. **两处他片期望更新——追认**：`anonymous_superclass_refuses_unproved_local_declaration_sites`（子方法完整→匿名折叠发生，编译运行已验证）与 null-check 片 discarded-construction 测试（改名、成员形边界逐字保持）——均为断言更新非删除，档案在 `05-corpus-delta.md`。
3. `chained` 保持拒绝（getfield 类初始化效果）为登记遗留边界。

## 残余

- 语句位+实参真实调用形（CST 域）与 `chained` 形登记在案；成员类语句位的池拼写归池拼写债域。
