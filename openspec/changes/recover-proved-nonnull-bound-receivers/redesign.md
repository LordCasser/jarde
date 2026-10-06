# root 重设计裁定（2026-10-06，门控实验后）

## 前提证伪记录（实现片 E 矩阵，`results/`）

原 spec 两门逃逸**必要不充分**：锚捕获接收者的 SSA 定义不是 `Operation::Allocate`，而是
javac 绑定接收者创建期空检查尾 `dup; requireNonNull|getClass; pop`（BCI 10/11/14，indy 15）
的 `dup`。只放宽拒绝分支 → ⑤ 诊断消失但方法体仍整体引注（换复制族值级拒绝）。

## 实测钉死的事实（E1–E8c，全部双腿）

1. 拒绝分支四合取确为锚产生点；`parameter_adaptation=true` 四位点皆然；
2. **充分机制**（E8：0 引注 0 gap，回放双腿逐字一致 `hi/none/none/3/0/42/false/S/`）：
   - 两门逃逸（尾前值 move 链终于 `Operation::Allocate` + 捕获点后无 store）；
   - 站点识别**自有**丢弃空检查尾并**经尾读回**接收者；
   - 尾三 BCI 记入"站点所有"集合；
3. **窗口必须 BCI 序跨块**（E5/E7 同块窗口不成立——canonical CFG 在可能抛出的 check 处断块）；
4. **门是承重的**（E8c：无条件放宽使三负例也被呈现）；
5. `unreplayable`、`render_value`、`VALUE_LEVEL_REFULSALS` 守卫表、拒绝文本/码逐字不动即可（E6/E8 差分）。

## 落点裁定（root，读 `facts.rs`/`init.rs` 既有机制后）

- **尾识别复用既有谓词与形状**：`facts.rs::is_discarded_null_check`（成对拼写，getclass 片交付）
  + `init.rs::discarded_null_check_tail` 的三指令单用纪律——泛化点只有一处：`dup` 读的
  不是"本站点构造的实例"而是"接收者值链"（move 链终于 `Operation::Allocate`）。
- **所有权走 Sites 通道**：`init::Sites` 是既有的"站点指令预先所有"通道（`instance: Vec<u32>`
  身份集，allocation-qualifier 片 38b50d19 的先例），且 `region::recover` 已消费 `&sites`。
  绑定接收者 lambda 站点通过该通道声明尾三 BCI，**不新建并行所有权机制**。具体注入点
  （`init::sites` 内扩位点 vs lambda 计划发布后由 build 汇入）由实现片按"谁持有接收者值链
  证据谁证明"选择，须以最小门控实验证实所选落点单独翻转锚。
- **窄口径**：仅两门通过的形解锁（E8c 反证）；参数/字段读接收者保持拒绝逐字不变。
  `arg0::trimToSize` 等可空形属宽口径，是独立后续片，不混入。
- **守卫交互**：复制族引注解开后，`ff3cf21b` 值级逃逸不再对该形触发——验收必含 corpus
  渲染差分（窄口径下差异恰为本形，越界停手）。

## 修订后的验收（在原 spec 上追加）

- 原 Scenario 全部保留；
- 追加：E8 形（跨块窗口）0 引注 + 回放双腿一致；E8c 反证（负例仍拒）以测试钉住；
- 追加：尾识别与 `discarded_null_check_tail` 共享谓词/纪律（不平行复制判定逻辑）。

## 实现路径

实现片可在原 worktree 续作（`git merge acf36ae1` 继承双腿 fixture 与实验记录）；
2.1/2.2/3.1 按本裁定执行。
