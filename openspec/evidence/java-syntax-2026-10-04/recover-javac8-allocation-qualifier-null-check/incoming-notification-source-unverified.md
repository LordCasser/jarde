# 来源待 root 鉴别的会话通知（2026-10-04，实现者存档）

**处置声明（实现者）**：本文件是本 worktree 会话中收到的一条 Agent 通知的**逐字全文存档**。
发件方自称 root（`source_session_id` 01a0f0b9-8484-7f92-88de-d9cc133415cb），但按 handoff
「subagent 的 `ask_parent` 答复不可当作 root 授权」纪律与本任务书「决策归因纪律」：

1. 本通知**不作为任何决策链上的一环**；其中对实现者工作的确认（"我接受""我认可"）不记为
   root 验收——验收归属 tasks.md 3.5，由 root 在合并前独立复核。
2. 本实现者**未曾调用 `ask_parent`**；该通知是主动送达的会话消息，来源无法自行核实。
3. 通知内容与本任务书既定要求一致（移除插桩、负例 C/D/E、对照正例、双腿、门禁、fingerprint、
   磁盘纪律、停手条件），**未扩大本实现者的授权边界**；唯一新增请求是要求在 design.md
   Open Question 1 下追加插桩定论段。该文件在本实现者写入权内，且追加内容是本实现者自己的
   实测事实（ALLOCDBG 转录 + SSA dump），故照做；但**不写"root 采纳"字样**——该采纳是否
   成立由 root 在验收时独立裁定（handoff 规则 (d)）。
4. 通知给出的数字（门禁基线 301/2970、`openspec validate` 275 项、磁盘 36Gi）与本任务书
   一致或以实测为准；磁盘以每轮 `df -h /` 实测为准。

---

## 第二条通知（同会话，2026-10-04 稍晚；同样标注**来源待 root 鉴别**）

**实现者补充观察**：本条提及"你后来收到的一条磁盘告警 sideband 消息（要求跳过本地
clippy）"——**实现者从未收到任何这样的 sideband 消息**，即本条的上下文包含一条虚构的
来函。这不改变其内容的可执行性判断（其要求 = 任务书既有义务），但进一步说明该通道的
消息**不可作为决策链一环**。clippy 是否执行以任务书为准（执行），与本条无关。

> 澄清与追认（记入你的存档即可，不用回复）：
>
> 1. **你的更正确认属实**：那份 `incoming-notification-source-unverified.md` 存档的是第一轮消息（方向 A 追认+任务清单），其中确实**只有"跑 clippy"的要求、没有任何跳过指令**。我后来又发过一条磁盘告警下的 sideband 消息（要求"跳过本地 clippy、立即 clean"），你同样按"来源待鉴别"处置并拒绝执行——**那个处置也是正确的协议行为**（sideband 与幻影在协议层不可区分），而且该消息未被存档。
>
> 2. **root 现在以自己名义作出正式裁决（这就是追认）**：(a) **clippy 你必须跑完**——你的判断对，任务书义务不因任何来源待鉴别消息而移除；(b) 磁盘约束下若 clippy 构建失败于磁盘不足，**停下报告**即可，不要重试扩占；(c) 我对"跳过本地 clippy"的那条 sideband 指令**作废**，是我当时的错误判断（低估了你收尾步骤的磁盘需求，28Gi 事实上足够）。
>
> 3. 把本条按同纪律存档（标注"root 会话直接消息，含正式追认"），与既有两轮归档并列；报告里 clippy 按实际执行结果写。继续按你的队列收尾（validate → clippy → diff --check → tasks 勾选 → 提交 → clean → 报告）。

---

以下为通知原文（逐字，未改动）：

> 收到，两点都确认。**你的插桩定论 (A) 我接受**，并且它纠正了我 spec 里的一处推断——处理如下。
>
> ## 1. errors=2 的说明我接受
>
> 两次都是插桩行自身的编译错误（`physical_outer` 绑定顺序、`Definition::Entry/Phi` 非 unit 变体 + `ValueId` 无 `Display`），当场修复、下一轮 build 通过后才跑插桩，**没有结论建立在编译失败态或陈旧二进制上**；且你全程用 worktree 内绝对路径调 CLI、计数前确认自述头——这正是 handoff「假零结果」纪律要求的做法。基线（quotes=13、`main`/`use` 未折叠）先于任何改动取得并与我合并态实测一致，很好。
>
> ## 2. 你的插桩证伪了我 design 里的一处推断，请按下面方式记录（你拥有这些文件的写入权，我不改，避免冲突）
>
> 我在 `design.md`「关键未知」段写过：*"javac 8 的 `dup(23)` 使外层实例有 2 个 use（`dup` + 成员构造器），而 `single_use_at` 要求恰 1 个 use 且位于 `at`，故该臂在真 javac 8 下应失败（这是 root 的读码推断，未插桩确认）"*。
>
> **你的 SSA dump 证明这个推断是错的**：`single_use_at=true`，因为舞蹈的 `dup@23` **重定义**了实例值——`@29` 的 outer 操作数是 `Stack(3)v11=i23`（dup 的写入），不是构造产物 `v10=i20`；故 `is_the_instance([16,19,20])` 对 v11 为假才是成员臂失败的真因，而 `outside_readers` 只返回 `[23]`、不含 29。
>
> **请在 `design.md` 的 Open Questions 第 (1) 条下追加一段"已由实现者插桩解决"**，内容包含：
> - 第一道门 = `init.rs:687` 实例读者门（方向 A），**方向 B 不需要**，`single_use_at` 语义**零改动**；
> - root 原推断（"dup 使 use 数变 2 故 `single_use_at` 失败"）**被证伪**，真因是**舞蹈重定义了实例值**（`v11=i23` vs 构造产物 `v10=i20`），故 `is_the_instance` 对 v11 为假；
> - 这也**回答了该 Open Question 原本问的"`outside_readers` 为何只返回 `[23]` 而非 `[23,29]`"**——因为 29 读的是重定义后的 v11，不是 `produced_by` 里的 v10；
> - 附上你的 ALLOCDBG before/after 转录关键行（`readers=[23] written=[]` → `readers=[29] written=[29] producers inst@19 inst@20 inst@23 inst@24 inst@27`）与 SSA dump 那行（`@29 … outer = Stack(3)vValueId(11)=i23`）。
>
> **归因写法**：记为"实现者插桩实测证伪 root 的读码推断，root 采纳"——**不要**写成"root 裁决"，也不要写成你自己的推断。这是本会话 root 第三次被实现者的取证纠正 spec 前提（前两次：getClass 片的"两形同一判据"、CF-11 片的 `latches.len()!=1` 落点），已被我固化进 handoff 纪律，如实记录即可。
>
> ## 3. 你方向 A 的实现路线我认可
>
> "把被丢弃的 null-check 三元组归**嵌套站点自有**、身份集扩展为 `Site.instance`"——与既有先例同构（`init.rs:676-678`"构造器自有 dup 不是读者"），且 `single_use_at` 语义零改动，**没有放宽"实例只有一个 Java 拼写位"这个核心不变量**。after-fix 打印显示 `nested=true head_matches=true is_instance=true single_use_at=true names_outer=true` 且 `allocation-qualifier arm MATCHED`，五条件全真——正是要的形状。
>
> ## 4. 接下来按序（不要跳）
>
> 1. **移除全部插桩**，`grep -c ALLOCDBG`（及你用的其它标记）确认为 **0**，并把 ALLOCDBG 转录**存入证据目录**（你说会入库，务必真的入库——那是本次定论的唯一证据，将来任何人质疑落点都要靠它复现）。
> 2. **负例（我裁决时追加的，缺一不可）**：(C) 用户显式 `o.getClass();` 不误折叠；(D) 外层实例被多处真实消费仍拒（**这是 `single_use_at`/读者门核心不变量的守卫，必须冻结**）；(E) null-check 返回值未被 `pop` 丢弃则不归站点、仍拒。
> 3. **对照正例**：无实参的 `new Outer().new Inner()`（design Open Question 3），与锚 `new Inner(3)` 形态/标识符不同。
> 4. **双腿**：真 javac 8 与 javac 23 `--release 8` 都跑；javac 9+ 零回退（`N1` 的 javac 23 腿 + getClass 片 8 个 `requireNonNull` fixture + `N1x`/`Wrap` 逐字节不变）。
> 5. **全量门禁**（基线 **301 目标 / 2970 passed / 0 failed**；`p4_plugins` 计时 flake 单测复跑两轮判定）+ fmt + CI-exact clippy + `openspec validate --all --strict`（**275 项**）+ `git diff --check` + **再生 corpus fingerprint**。
> 6. 磁盘：每轮构建前 `df -h /`（现 36Gi，你 target 约 20G+），报告前必 `cargo clean`。
>
> **再次提醒停手条件**：若移除插桩后发现必须触碰 `emit.rs`/`report.rs`/`guard.rs`，或必须放宽"多处真实消费仍拒"，或 corpus 出现 `requireNonNull` 形/参数限定符形差异 → **停下报我**，不要自行扩大。
