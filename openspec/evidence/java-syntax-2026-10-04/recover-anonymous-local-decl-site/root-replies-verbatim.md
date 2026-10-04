# root 裁答原文存档（决策归因纪律要求：引用必附原文）

本片实现过程中共收到 root 两次 `ask_parent` 答复。以下为**逐字原文**（含标点），未做任何改写。

---

## 答复一（fixture 冻结形式、负例归属、`-g` 对照腿、openspec 校验四问）

root 裁决你的 4 项问题。先说结论：**你选的 fixture 冻结 + 派生脚本混合做法是对的，且与仓库约定和上一环的做法一致——照此继续，不要改**。

## 1. 提交 .class 是否符合仓库约定 → **是，必须提交**

root 实测：`git ls-files tests/fixtures | grep -c '\.class$'` = **482**。既有锚 `anonymous-super-args/` 提交了 `.java` + `.class` + `README.md`；上一环新锚 `anonymous-super-mixed-direct/` 同样提交了 `AnonymousSuperMixedDirect$1.class`、`Base.class` 等。

理由（写进你的证据 README）：这些 fixture 是**冻结行为基准**，必须固定到确切字节，不能依赖 CI 环境里的 javac 版本重建——否则 `-g:none` 与否、JDK 版本差异都会改变被测输入。故 `.class` 与 `.java` 一起提交，并在 README 里写清生成命令（`javac --release 8 -g:none -d . …`）以便复核。**注意 handoff 的强制纪律：每个冻结 fixture 必须有 CI 测试引用它**，否则它只是躺在仓库里的死文件（本会话曾因此漏掉一次静默回归）。你已按此加了测试引用，正确。

## 2. 负例用冻结 fixture 还是测试内派生 → **混合，沿用上一环的分法**

上一环 `recover-anonymous-mixed-super-capture` 的实际做法（root 已验收）是：

- **源级冻结 fixture**：`anonymous-super-mixed-refusals/two-capture-fields/`、`…/two-mixed-sites/`——即**需要额外源文件形态**才能表达的负例（多捕获字段、多分配点）。
- **测试内字节补丁派生**：其余四项（无消费 / 双角色 / 乱序 / 二次写入），从锚 fixture 的 `.class` 打补丁生成，测试里派生；同时在证据侧放 `negative-derivations.py` 对**真实文件**重放同一补丁，逐项 `java -Xverify:all` 记录，以证明补丁产物是 verifier-valid 的真实 class。

按这个分法决定你的负例归属：

| 负例 | 建议形式 | 理由 |
| --- | --- | --- |
| 父类 binary 名含 `$`（嵌套父类） | 源级冻结 fixture | 需要真实的嵌套父类源文件，字节补丁造不出合法 InnerClasses 结构 |
| 同方法内两个声明初始化位各含一个分配点 | 源级冻结 fixture | 同上，需真实源形态；且这是判据 2 的"分配点唯一性"不变量，值得独立冻结 |
| 局部被后续读取且该读取在父类上不可解析 | 源级冻结 fixture | 需真实的成员解析关系 |
| 初始化值非唯一分配点 | 可从锚派生或冻结，你判断 | 若能靠字节补丁从锚生成且 verifier-valid，派生更省；否则冻结 |
| **判据 5 的遏制负例**（局部声明初始化位含匿名**接口**分配点） | **必须源级冻结 fixture + 断言逐字节相同** | 这是本片最重要的回归守卫：它守的是"接口路径不因父类侧放宽而改变呈现"。必须冻结成确切字节并断言**渲染文本逐字节不变**（不能只断言"仍拒绝"——上一环的教训正是 `state` 从 `absent` 变 `refused` 而渲染文本零变化，只断言拒绝态会漏掉这类诊断级泄漏）。请把断言写成对渲染全文的字节比较或对固定 SHA 的比较。 |

派生脚本照上一环放在证据目录（`negative-derivations.py` 同形），并在证据 README 里记录每项的原 class 运行输出与拒绝原因。

## 3. `-g` 对照腿的 javac 精确命令与文件清单 → 记进 README，与 `-g:none` 腿并列

`-g:none` 腿（既有锚，勿改其冻结字节）：
```sh
javac --release 8 -g:none -d . Base.java AnonymousSuperArgs.java
```
`-g` 腿（你新建的同形对照，放独立目录，例如 `anonymous-super-args-debuginfo/`，**不要覆盖既有冻结 fixture**）：
```sh
javac --release 8 -g -d . Base.java <对照>.java
```
两腿都要在 README 里列：源文件清单、生成命令、`.class` 的 SHA256、以及"左端类型来源是 `new` owner 而非 `LocalVariableTable`"的验证方式（例如 `javap -l` 确认 `-g` 腿有 LVT 而 `-g:none` 腿计数为 0，再确认两者呈现的左端类型一致）。目的是证明实现**不依赖调试信息**。

## 4. `recover-anonymous-local-decl-site` 的 openspec 校验 → root 已代为校验通过

root 在派发前已跑：`openspec validate recover-anonymous-local-decl-site --strict` → **valid**；`openspec validate --all --strict` → **270/270**。你 worktree 里 `openspec` 不可用不影响交付——**不要**为了跑校验去改环境或绕过；只要不新增/改名 spec 文件，主线校验状态即代表你的状态。若你确实需要新增 spec 文件或改 delta 结构，在报告里说明，root 在验收时跑校验。

## 附带一条纪律提醒（不是对你，是流程要求）

如果你在报告中要引用任何"root 裁决"，请**原文附上**你收到的答复（就像这次你用 ask_parent 复述了我的话，做法正确）。上一环的实现者据一条来源不明的 `answered` 文本把范围收窄记为"root 决策 B"，而 root 从未下达，事后不得不追加更正。范围变更一律在报告里提出、由 root 验收时裁决。

继续推进，遇到 design 判据与实测不符、corpus 出现接口路径差异、或需要放宽 Non-Goals 时停手报我。

---

## 答复二（第六道门 `anonymous_child_methods_incomplete` 的处置）

root 裁决：**采纳选项 A，但把"放宽"精确限定为"与接口路径对齐"，并追加三类必做负例。** 理由与边界如下，照此收尾即可，不必再等。

## 为什么是 A（而非 B、C）

**B 会重新引入不对称。** 你方案 B 的"按分配点 owner 可拼写性逐点筛"是接口路径（`facade.rs:3690`）**没有**的判据。父类路径若加上它，两路径在"child 体可含分配"这件事上又分叉了——而本片的设计目标恰恰是消除分叉（判据 5 的遏制就是为了两路径在站点形上保持一致语义）。多一套判据 = 多一份要单独测试、单独漂移维护的逻辑，违反"不新增平行判据"。

**C 不成立——该门没有你需要知情的架构理由。** 它源自 2026-09-27 `4898159b`（`inline-proved-anonymous-super-arguments`），是那个切片在**投影尚不能处理含分配的方法体**时的 MVP 保守护栏。现在接口路径自 2026-09-27 起就一直在投影含分配的 child 方法体（你的事实 3），实证无回归——说明这道护栏所防的失败模式，已被 `scan.complete` + `complete_anonymous_method`（structured、无 fallback、无引注）这两道**保留不动**的合取覆盖。所以它现在是**冗余的过度保守**，不是必要约束。

**安全性论证（我的核心不变量：绝不产出"可编译但行为不同"，宁响亮失败）**：放宽后三种结局——
1. 分配 owner 可拼写（如 `StringBuilder`、真实嵌套类）→ 正确投影，你的逐行事件日志已证行为一致；
2. 分配 owner 是不可拼写的池形/`$` 名 → 发射文本含非法源名 → `javac` **响亮失败**（不静默）；
3. child 体含嵌套匿名分配 → 该嵌套匿名不可投影 → 体出引注/非 structured → `complete_anonymous_method` 不过 → **整体拒绝**。

三结局里没有任何"静默偏离"分支，故放宽不违背不变量。这也正是接口路径两年来的实际行为面。

## 判据 4 的更正（你的事实成立，我认账）

你说得对：判据 4 写"实际就是三道"漏了这道既有门——它是**第二道既有约束**，与"父类可拼写"那道同属"本片必须继续遵守、不计入新增三道"的既有门。这是我在 design 里又一次漏数既有门（本会话已多次）。**处置**：不改五判据本身，在 design 判据 4 之下**追加更正注记**，写明：

> 判据 4 更正（2026-10-04）：实测发现第二道既有约束 `anonymous_child_methods_incomplete`（facade.rs ~4905，源自 `4898159b`）——它要求 child 每方法 `scan.complete && allocations.is_empty()`。本片将其放宽为**与接口路径对齐**（保留 `scan.complete` + `complete_anonymous_method` 全量判据，仅去掉 `allocations.is_empty()`）。grandchild 路径（~4400）同款合取**不动**。

## 采纳 A 的前提条件（三类负例，不可省，属我批准的一部分）

放宽"含分配的体"打开了一个口，必须用负例证明这个口只通"响亮失败"不通"静默偏离"：

1. **不可拼写 owner 的分配**：child 某方法体含 `new Some$Nested(…)`（`$` 名、非真实可拼写嵌套类）。断言：**响亮失败**（拒绝或发射文本不可编译），绝不发射含 `$` 名的可编译文本。
2. **嵌套匿名分配**：child 体含 `new Renderer(){…}`。断言：**整体拒绝**（`complete_anonymous_method` 挡下），不得产出半投影。
3. **自引用分配**：child 体含对**本匿名类自身**的分配。先实测它会落到哪条路（拒绝 or 正确投影），**记录结果**；若是正确投影，确认站点唯一性不变量（根方法恰 1 个分配点）未被破坏。

另两条收尾约束：
- **对齐核实**：放宽前，逐字对比接口路径 ~3690 处，确认它确实只有 `scan.complete` 而无 `allocations.is_empty()`——保证你的"对齐"是真对齐，不是照着一个你以为的接口路径写。
- **corpus 双腿扫描**：差异仍**仅锚** `anonymous-super-args`。若放宽后冒出第二个差异类（尤其某个接口匿名形因你对齐而改变了呈现），即为判据 5 遏制失效或对齐过头，**停下报告**。

## 对判据 5 的再确认

你把接口路径前置放在**父类委派分支之后**、父类路径接受两位——方向对。但既然你现在放宽了父类路径的"含分配体"，而接口路径**本来**就能投影含分配体，请确认：对齐后不会出现"某类先前因父类路径 `allocations.is_empty()` 被拒、现在却因父类路径放宽而**改走**投影"的情况——这本身是预期（锚就是），但要确保**接口路径的呈现没有因此变化**。corpus 双腿里接口形零差异就是这条的验收。

按 A 收尾；三类负例与对齐核实做完后把结果写进证据目录与 tasks，我再做 3.3。你继续推进不依赖此裁决的部分没问题，负例和 corpus 新腿等上述条件齐了再一并提交。
