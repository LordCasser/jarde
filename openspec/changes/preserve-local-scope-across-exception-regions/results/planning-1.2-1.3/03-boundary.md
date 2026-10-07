# 1.2/1.3 切片：锚 12/13 的边界（re-slice 依据）

gating 实验（`02-gating.md`）证明：**声明规划不是锚 12/13 的 gate**。本文件记录 gate 在哪、缺什么。

## 1. 锚 12 `LK.take()I` 的完整证据链

javap（`[7,50) → 59 any`，**只有一行**，没有 self-protection 行；保护体内是循环
`7→11→14/26`，正常出口 `49: istore_1` 保存返回值 → `50-54` 解锁 → `57: iload_1; 58: ireturn`；
handler `59: astore_2` → `60-64` 解锁 → `67-68` 重抛）：

1. region walk 走到保护区的第一个块 BCI 7：`leaving_edge` 给出 `FallbackReason::ExceptionEdge`
   （`region.rs:6663`），而 `frame.own_try == Some(node) && handlers.len() == 1 &&
   handlers[0].catch_type_index.is_none()` 时 **`guard::examine` 被跳过**（`region.rs:2732-2755`）；
2. 于是先试 `guard::shared_finally_candidate`（`region.rs:2596-2612`）；它的 1 行分支要求
   `row.start_bci == 0 && current.bci() == 0`（`guard.rs:9835-9843`）——`take()` 的行从 7 开始，返回 `None`；
3. 然后 `guard::guarded`（`guard.rs:12121`）只试“已认证形状”表；`handlers.len() == 1` 时只调
   `prove_conditional_finally`（`guard.rs:9883-9885`）——不是本形状，`None`；
4. 唯一的一般性“两份清理副本 + 独占完成形”证明是 `prove_finally_copy`（`guard.rs:2836`），但
   * 生产调用点只有 `resources()`（`guard.rs:13932`，TWR 路径）；
   * 它明文拒绝“行起点之前有会抛出的调用”（`guard.rs:2849-2852`）——`take()` 的 BCI 4 是 `lock.lock()`；
   * 它只证直体后缀（一份块内、无分支）。

即便 (4) 通过，region 层还需要：`finally_body`（`region.rs:5067`）走的 `finally_body_supported`
（`region.rs:2400`）**不接受 `Region::Loop`**，所以“保护体内含循环”的 finally 体需要自己的 region reader
（对照 `LoopFinally`/`MultiReturnLoopFinally` 各自的 `*_regions` 构造）；而 `SavedReturn` 的保存值写在
**体内**（不是 lead）时，声明规划/builder 还要为这个保存局部提供提升与返回呈现（现有证书
`flag_saved_return`/`local_null_saved_return`/`null_lead_straight` 都是固定形状的，`build.rs:1004-1137`）。

`LK.tryLockQuick()Z` 是同一族的另一形：`if` 规则找到 finally-copy 候选后以 `Unproven::FinallyCopy`
拒绝（`guard.rs:13936` 的 `Verdict::refused(None, Unproven::FinallyCopy, at)`）——候选缺的是
straight-body/copy/range/ownership 四件套证明。

## 2. 锚 13 `IO.countLines/readAll` 的额外机器

两方法各 2 行（`[25,45)→52, [52,54)→52` / `[17,37)→44, [44,46)→44`）：2 行分派表
（`guard.rs:9849-9897`）试遍 `loop_finally`/`nullable_resource_finally`/`flag_conditional_finally`/
`local_null_conditional_finally`/`segmented_null_lead_finally`/`empty_catch_call_finally`/
`nested_join_finally` 后仍无匹配；保护体是“循环 + 会抛出的调用 + 保存返回值”，与锚 12 同因。

除同一 guard 证书外，锚 13 的**整类验收**还差两个别的族的位点（巡查 README 已预记）：
`IO.main` 的 `String → java.lang.CharSequence`（family-6 宽化），以及 `countLines` 三层 ctor 链的
`jre_new_shape`（“the construction at BCI 8 completes inside the construction at BCI 0 …”）。
这两项不属于本 change。

## 3. re-slice 需要的最小面（建议）

一个 guard/region 层切片（不是声明规划层）：

1. 一般 `try { … } finally { … }` 证书：以 `prove_finally_copy` 的副本比较为基础，放宽“行起点前的调用”
   边界（行起点是异常表自己声明的，不是推断的），并把完成形扩到 `Void`/`SavedReturn`/重抛三形；
2. 允许保护体含控制流的 region reader（`finally_body_supported` 或专用构造器）；
3. 保存局部写在体内时，声明规划/builder 的提升与返回呈现；
4. 验收按锚的要求：双 leg javac（`--release 8` + 真 javac 8）+ `-Xverify:all` + 输出逐行一致
   （`LK.main` 的 `lock/unlock` 顺序即其自身探针）。

本切片**未**实现上述任何一项（任务书的 “do not force scope”）；`tests/preserve_local_scope_plan.rs`
的负例把这批形状的现状逐字钉住，re-slice 落地时那两条负例是**需要显式更新**的对照。
