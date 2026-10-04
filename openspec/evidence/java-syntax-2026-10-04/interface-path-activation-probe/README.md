# 共享站点扫描的接口路径激活取证（2026-10-04，root）

**背景**：`recover-anonymous-mixed-super-capture`（合并 `e1c89d57`）为支持"前导 `Declare` + 末条直返"而放宽了 `crates/jarde-java/src/report.rs::class_source_direct_return_new`。root 读码发现该 helper 位于**接口匿名路径与父类匿名路径共享**的站点扫描链上，故放宽可能顺带激活接口路径——而那不在该片的 Non-Goals 内、也无人取证。本文件记录 root 的实测结论。

**结论：激活确实发生，但当前被一个 *既有且非专门设计* 的写回门挡住，故渲染文本零变化（响亮失败），属**诊断字段级**的范围泄漏而非行为偏离。** 这印证了 [recover-anonymous-local-decl-site/design.md](../../../changes/recover-anonymous-local-decl-site/design.md) 判据 5 要求**显式遏制**的必要性——本片（local-decl-site）正要扩展写回能力，一旦写回门能处理更多语句形，接口路径就会**静默激活**。

## 调用链（root 读码确认，非推理）

```text
class_source_direct_return_new (report.rs:6596)        ← mixed-super-capture 放宽点
  └─ class_source_anonymous_return_site (report.rs:533)
       └─ facade.rs:8885-8917 循环全部 method_asts 累加 → _anonymous_return_sites
            └─ facade.rs:2091 前置 _anonymous_return_sites.len() == 1
                 └─ project_class_source_anonymous_interface (facade.rs:3293，解构 [site])
                      └─ 3367 委派 project_class_source_anonymous_super
```

两路径消费**同一个**站点向量；接口投影在前，并在 3367 委派父类投影。

## 三形实测（主线合并后二进制，`javac --release 8 -g:none`，顶层接口以避开 `$` 门）

| 形 | 源 | `anonymous_interface_projection.state` | `make()` 呈现 | 性质 |
| --- | --- | --- | --- | --- |
| **纯直返**（无参、返回恰为 `()LTopI;`、末条 `return new TopI(){…}`） | [IP3.java](fixture/IP3.java) | **`projected`** | `return new TopI() { public java.lang.String go() { return "anon"; } };` | 已验收能力，正确 |
| **前导 Declare + 末条直返**（前导声明带可观察副作用 `String s = side();`） | [IP2.java](fixture/IP2.java) | **`refused`** | `java.lang.String local0 = side(); return new IP2$1();`（物理文本） | **写回门拒绝** |
| **嵌套接口**（`IP$I`，binary 名含 `$`） | [IP.java](fixture/IP.java) | `absent` | 物理文本 | 既有 `$` 门先拒，未达写回门 |

**IP2 的拒绝原因（报告字段原文）**：

```text
anonymous_interface_projection.reason = "unsupported (anonymous_root_writer_rejected): the class-source writer cannot reproduce the root around this m…"
```

落点 `src/facade.rs:3967-3968`：`class_source::source_text_with_method_projections(...)` 返回 `None` 时拒绝。该门是**通用写回能力门**，不是针对"前导声明形"的专门遏制。

**关键安全事实（root 实测）**：IP2 的渲染文本**完整保留了前导声明**（`java.lang.String local0 = side();` 在 `return` 之前）与 `main` 的两条 `println`；完整源集 `javac --release 8` **exit 1 `找不到符号`**（`new IP2$1()` 的 `$1` 非合法标识符）——**响亮失败，无静默偏离**。原 class 基线为 `anon` / `side-effect-ran`（[results/IP2-original.out](results/IP2-original.out)），前导副作用可观察。

## 因果链（代码级推断 + 实测端点；root 未重建基线二进制，据实登记证据界限）

- **放宽前**：`class_source_direct_return_new` 要求 `program.stmts.len() == 1`，IP2 形有 2 条语句 → 返回 `None` → 不产站点 → `_anonymous_return_sites.len() == 0` → `facade.rs:2091` 前置不成立 → **接口投影根本不被尝试** → `state` 保持默认 `absent`。
- **放宽后**：IP2 形匹配"前导全 `Declare` + 末条直返" → 产站点 → `len() == 1` → **接口投影被尝试** → 写回门拒绝 → `state` 变为 `refused`。
- 实测端点已确认（IP2 = `refused`、IP3 = `projected`、IP = `absent`）；"放宽前为 `absent`"这一步是**代码级推断**，root 未重建 `996ac2b7` 基线二进制复测（其 worktree 已在验收后移除，重建需一轮全量编译）。推断依据是 `state` 的默认值与 2091 前置的合取结构，链条无歧义，但按证据界限如实标注。

## 影响评估

- **渲染文本（交付物）**：零变化。IP2 前后都是物理文本 + 响亮编译失败。这解释了 mixed-super-capture 的 corpus 双腿扫描为何报"49 渲染 / 仅 1 处差异（新锚）"——**该扫描只比对渲染文本，不比对报告 JSON**，故诊断字段变化不在其覆盖内。
- **报告 JSON（诊断产物）**：接口匿名形的 `anonymous_interface_projection.state` 由 `absent` 变为 `refused`。属**范围泄漏**（该片 Non-Goals 未含接口路径），但无行为后果。
- **对 `recover-anonymous-local-decl-site` 的风险（本片取证的真正动机）**：该片的判据 3 要求扩展 emitter/写回能力以重拼赋值左端类型。**一旦写回门能处理"前导声明 + 分配点"这类形，IP2 形就会从 `refused` 变为 `projected`**——即接口匿名投影在**无人取证、无专门测试**的情况下被激活。届时若接口路径的捕获判据或左端重拼与父类路径不同（例如接口路径的 `captured_root_parameter` 分支在 `facade.rs:3545-3549` 有独立的 `(D)L…;` 描述符形态），就可能产出**可编译但行为不同**的文本。

## 处置

1. **已在 [local-decl-site/design.md](../../../changes/recover-anonymous-local-decl-site/design.md) 判据 5 钉死显式遏制方案**：站点元组增加 `AnonymousSiteShape::{DirectReturn, LocalDeclInitializer}` 判别位，**接口路径前置改为要求 `DirectReturn`**（与放宽前逐字节等价），父类路径接受两位；并强制一个"局部声明初始化位含匿名**接口**分配点"的冻结负例，断言实现后呈现逐字节相同。
2. **不得走 design 判据 5 的方案 3**（顺带取证接口路径）——那会使本片范围翻倍并与 `recover-proved-anonymous-local-capture`(6/6) 重叠。
3. **登记为独立债务（不阻塞本片）**：mixed-super-capture 已造成的诊断字段变化（`absent`→`refused`）应补一个冻结负例守住"接口路径不因父类侧放宽而改变呈现"；corpus 双腿扫描宜增加**报告 JSON 的投影状态字段**比对，否则此类诊断级泄漏无法被该扫描发现。

固定转录见 [fixture](fixture/)（`IP.java`/`IP2.java`/`IP3.java`/`TopI.java` 及其 `.class`、`fam.jar`）与 [results](results/)（三份渲染全文、原 class 运行输出、SHA256）。原 class 为行为基准（`anon` / `side-effect-ran`）。
