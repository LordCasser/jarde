# Root 独立验收（2026-10-07，1.2/1.3 测试面交付，合并）

## 裁定

1. **停手处置正确**：门控实验（`02-gating.md`）证明锚 12/13 的拒绝链在 region/guard 层——`LK.take()` 撞 `FallbackReason::ExceptionEdge`（BCI 7 保护块首）+ `shared_finally_candidate` 的行起点==0 前置 + `prove_finally_copy` 的行前调用拒绝；`IO` 撞 2 行分派表全 miss。声明规划是**下游**（region 树是输入），分类改动单独翻不了任何锚——1.2/1.3 保持未勾正确。
2. **交付价值追认**：三分类落点图谱（Local 873/983、Elevated 1219+1836、Incomplete 六处、子作用域 3412/789）+ 681 行行为钉面（词法 owner 无泄漏/可提升带逐 region source-map origin/不完整逐字拒绝/停止契约）是 1.2/1.3 的**测试验收面**，后续 guard 片落地后即为现成回归网；census/fingerprint 更新为纯新增。
3. **root 实测**：`preserve_local_scope_plan` 5/5、oracle ignored 3/3、全量 **335 targets ok / 0 FAILED**、fmt/clippy/openspec 308/308 全绿；`04-roundtrip.sh` 双腿 `1,2,3,4,6,5,9,10,11`。
4. **re-slice 依据采纳**：`03-boundary.md` 的四件套清单（1 行前置/行前调用+直体限制/`finally_body_supported` 排斥 Loop/SavedReturn 体内写）构成立项依据——另立 `recover-lock-guard-loop-finally`。

## 账本

1.2/1.3 部分交付（测试面 + 落点图谱；验收锚转 `recover-lock-guard-loop-finally`，其落地后回填勾选）。
