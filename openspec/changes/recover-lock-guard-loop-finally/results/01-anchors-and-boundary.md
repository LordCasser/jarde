# 1.1 锚与证据链复核（HEAD，按名重验）

日期 2026-10-07。基线 = 父提交 `6f8ed47a`；本切片 = `8ca9a132`（生产实现）。

## 1. 四件套证据链（`03-boundary.md` 的四件）按名重验

行号为 HEAD（`8ca9a132`）实测；取证文档中的行号已漂移，锚点名称不变。

| # | 取证锚点 | HEAD 实测 | 复核结论 |
| --- | --- | --- | --- |
| 1 | `shared_finally_candidate` 单行分支的"行从 0 开始"前提 | `guard.rs:10291`（函数）；单行前提 `row.catch_type_index.is_some() \|\| row.start_bci != 0 \|\| current.bci() != 0 \|\| ops.get(row.end_bci) != Load{0} \|\| handler 非 Store` 紧随其后 | 成立：`LK.take/put` 的行从 BCI 7 开始 → 该前提拒绝；这是本切片新增分支**之前**的那道门 |
| 2 | `prove_finally_copy` 明文拒绝"行起点前的会抛调用"、只证直体后缀 | `guard.rs:2873`；`for before in facts.bcis((0, row.start_bci)) { ... if Invoke → None }` | 成立：`LK` 三法的 BCI 4 是 `lock()`/`tryLock()` → 拒绝；该函数与 `resources()` 生产路径**未被本切片触碰**（`git diff` 只新增，不改其判定） |
| 3 | `finally_body_supported` 不接受 `Region::Loop` | `region.rs:2408`（函数体未改） | 成立且**未被本切片改动**：新形状的 body reader 走 `bounded_shared_finally_body`（`region.rs:6410`），其循环许可谓词（`looping_body`）按 `Shape::LockGuardFinally` **新增一项**，既有形状的答案逐字不变 |
| 4 | `SavedReturn` 体内写形的提升呈现 | `build.rs:1092` 起的固定形状证书族（`flag_saved_return` 等） | 复核结论：本切片的 saved return **不需要**新增提升条件——见 §3 |

## 2. javap 转录：`LK` 三法的完整序（锚，`lk.jar`）

```
put()V   0 aload_0; 1 getfield lock; 4 invokevirtual lock()V          ← 行前唯一会抛调用（acquisition）
         7..41 保护体：7 aload_0; 8 getfield count; 11 iconst_2; 12 if_icmplt 27;
                15 aload_0; 16 getfield notFull; 19 invokeinterface await()V; 24 goto 7;
                27..34 count++; 37..41 signalAll()
         46 aload_0; 47 getfield lock; 50 invokevirtual unlock()V      ← normal copy
         53 goto 66; 56 astore_1                                       ← 完成（transfer）+ handler 绑定
         57 aload_0; 58 getfield lock; 61 invokevirtual unlock()V      ← handler copy
         64 aload_1; 65 athrow                                          ← 重抛同一值
         66 return
         Exception table: [7, 46) → 56 any                                ← 单行、无自保护行

take()I  0 aload_0; 1 getfield lock; 4 invokevirtual lock()V
         7..40 保护体：while (count <= 0) await(); count--; signalAll()
         45 aload_0; 46 getfield count; 49 istore_1                     ← 保存返回值（体内）
         50 aload_0; 51 getfield lock; 54 invokevirtual unlock()V       ← normal copy
         57 iload_1; 58 ireturn                                          ← 完成（SavedReturn）
         59 astore_2; 60..64 unlock()V; 67 aload_2; 68 athrow            ← handler
         Exception table: [7, 50) → 59 any

tryLockQuick()Z
         0 aload_0; 1 getfield lock; 4 invokevirtual tryLock()Z; 7 ifeq 42
         10..18 count += 10; 21 iconst_1; 22 istore_1                   ← 保存 true
         23..27 unlock()V; 30 iload_1; 31 ireturn                        ← normal copy + 完成
         32 astore_2; 33..37 unlock()V; 40 aload_2; 41 athrow
         42 iconst_0; 43 ireturn                                         ← if 的 else 臂
         Exception table: [10, 23) → 32 any
```

三法的共同形状：**单条 any 行**、**无自保护行**、acquisition 是行前唯一 invoke、两份 release 是同一三指令文法
（`aload_0; getfield lock; invokevirtual unlock()V`）、handler 恰为 `astore; <copy>; aload; athrow`、无后继。

## 3. 第 4 件（SavedReturn 体内写）在本切片为何不需要提升

`take`/`tryLockQuick` 的保存写在**体内**、读在体内同一 canonical block（`take` 49/57 同属块 `[26,59)`；
`tryLockQuick` 22/30 同属块 `[10,32)`）。该块由 body region（`bounded_shared_finally_body` 的 `Straight`）认领，
于是两次访问落在**同一 region path** → 规划给出 `Local { owner }`：保存**就地**写成
`int local1 = this.count;`（`build.rs` 的 `block()` 只在 `self.shared_finally.is_some()` 时跳过 save，
本形状不置该状态），完成由 `guarded_return(returns)` 写成 `return local1;`。
因此 `flag_saved_return` 一类的**提升**条件不需要新增；`declare_at`/`at_region` 不变。
（对照：Tf2 的 `Tf2$Result local3 = new Tf2$Result(400); return local3;` 是同一机制的既有实例。）

## 4. 新证书的形状判据（实现落点）

`guard.rs:3632` `prove_lock_guard_finally`（`lock_guard_copy` 在 `guard.rs:3554`），逐条读本 run 的事实：

1. 恰一行、catch-all、`row.start_bci == current.bci()`、handler 块无后继；
2. 行前**恰好一次 invoke**（acquisition），其接收者是实例字段读，且该字段在**整个方法**内无 `putfield`；
3. handler = `astore e; <copy>; aload e; athrow`，`handler_binding` 证绑定来自本行交付、重抛同一 SSA 值；
4. 两份 copy 都是 `aload S; getfield F; invoke M` 三指令文法，**载入同一 SSA 值**（acquisition 的字段读所读的那个
   定义，且该值是 `Definition::Entry`——方法自己的 `this` 或参数，不是体内写出的值），字段与目标相同；
5. 两份 copy 都在保护区外（release 若能被本行重入就会跑两次）、保护区内无 `Return`；
6. 保护区的每个正常出口都到 release（跨行尾的块其出口在 release 之后，由完成形证明约束）；
7. 完成形二选一：`SavedReturn { save, returns }`（store→load SSA 链）或 `Void { transfer, returns }`
   （transfer 唯一后继 = 方法自己的无值 `return`，无其它前驱）；
8. `owned` = 保护区块 + handler 块；`join` = `Void` 时是那个 `return` 块。

安全性来源一句话：两份 release 与 acquisition 读**同一字段、同一 SSA 接收者值**，且该字段全方法无写——
所以折叠出的一个 `finally` 在两条路径上都释放 acquisition 取到的那个对象，恰好一次。

## 5. 区域走序的一处必要调整（按证据说明）

`region.rs` 的 `region_at_inner` 中，guard 候选（`shared_finally_candidate`）原本排在 loop-header 分支**之后**：
保护体以循环开头时（`LK.put/take` 的行起点就是循环头），走序会先把它当循环呈现、于是证书永远问不到。
本切片把该候选块**整体前移**到 loop 分支之前——条件本身（`frame.own_try != Some(node) && frame.own_finally.is_none() && starts_catch`）逐字未改，
所以非 catch-start 的块走序不变；只有"既是循环头又是 catch-start"的块改为先问证书。
证据：`02-corpus-diff.out`（全 1987 类零移动）+ 测试面零回退。
