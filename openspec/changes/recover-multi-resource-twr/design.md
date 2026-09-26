## Context

自写 `Patrol.multiResource`（javac --release 8，无 ifnull 资源头）的异常表五行：

```
[11, 23) → 44  Throwable   内层资源的异常路径
[51, 56) → 59  Throwable   内层 close 自身的保护（addSuppressed）
[ 5, 33) → 71  Throwable   外层资源行（覆盖内层声明、体、内层正常 close）
[44, 71) → 71  Throwable   内层 close 链被外层再保护
[77, 81) → 84  Throwable   外层 close 自身的保护（addSuppressed）
```

`guard.rs::twr` 的层级链（innermost 起、严格包含、`closes_something`）能找出 [11,23)→44 与 [5,33)→71 两层，但几何校验要求「chain[i].end == handlers[i+1].span.1」（外层行止于内层 handler span 末），实际外层行止于 33（外层语句自身体末，内层正常 close 链之后）→ `Unproven::RangeEnd`。

仓库已有 `Guarded.two` 的旧正例采用另一种合法布局：内层 `[12,15)→26`，外层 `[6,46)→57`，内层 handler 为 `[26,46)`；外层原行直接保护内层异常清理，结束 BCI 46 同时是外层正常 close 的起点。`Patrol` 的内层 handler `[44,71)` 则由独立 `[44,71)→71` 行保护，且该行与外层 `[5,33)→71` 指向同一 handler、具同一 catch 类型。已有等式只碰巧覆盖第一种布局。`Patrol` 的体内计算结果先存入 slot 5，正常关闭后才 `iload 5; ireturn`（BCI 41、43）；若仅改几何，当前 Builder 仍可能把返回放在 `try` 外，失去局部变量作用域和源码返回语义。

## Goals / Non-Goals

**Goals:** 层级几何改为「每层行止于该层语句末」；一条 N 资源头的呈现；close 逆序与抑制链既有证明复用；两份冻结 fixture（release 8 无检查形状为主）。

**Non-Goals:** 单资源几何改动；ifnull 资源头新证明；TWR+多捕获；close 顺序放宽；把 TWR 失败降级为普通 try/catch。

## Decisions

1. **结束位置从正常 close 链读取。** 先逐层证明资源初始化、handler 的 close/suppression、正常路径逆序 close，并记录每层正常 close 组的起点；再要求该层主行的 `end_bci` 精确等于该起点。旧 `Guarded.two` 的外层 46 与新 `Patrol` 的外层 33 均满足，不能以 `inner.handler.span.1` 替代。主行仍须按原规则严格包含内层主行，且从该层资源初始化之后的第一条受保护指令开始；包含关系本身不授权额外字节。
2. **内层异常清理的外层保护闭合。** 对相邻两层，内层 handler 的完整 span 必须由外层主行覆盖；若它不在主行内，只准接受另一条范围**恰好**等于该 span、同 catch 类型且同目标 handler 的异常表行。此行连同原层行、close 自身 guard 行一起进入已解释行集合；记录表序与实际覆盖优先级，不得吞并用户 catch、部分重叠行或第三条未解释行。该检查不需要公开异常模型或第二套扫描器。
3. **返回仍写在资源体内。** 对冻结的 `Patrol` 返回形状，体内先产生并保存返回值，close 之后只有同一值的纯 load 和类型相符的 return；证明后让现有资源 `Plan` 携带该 return BCI，Builder 复用既有 `return_expr` 将 `return` 写在 `try` 体内并给原 load/return 留来源锚点。若尾部还有调用、写入或返回了别的值，拒绝而非把体内局部在体外读取。无返回或在 `try` 后另有语句的旧形状保持原路径。

   2.3a 的同次 CFG/SSA 证明与 Builder 接缝已实现。高层真实 fixture 的清理 handler 在正常 return 之后，曾因 `Plan.owned` 仅含 `start..return` 连续块而落入 Region fallback；`slot_uses` 又把 handler 的 caught Throwable 与体内保存的 int 按同一物理 slot 合并，导致词法声明拒绝。2.3b 已将逐指令可解释、入出边仅属于已证清理链的非连续 handler 块纳入 Guard ownership，并仅按这些已证 cleanup 指令 BCI 从源局部声明规划排除编译器读写；竞争异常表行仍拒绝，没有全方法跳过该 slot 或隐藏用户 handler。其它跨异常区局部身份仍属于独立 `preserve-local-scope-across-exception-regions` 任务。多资源冻结样例另有每个头的 `new; dup` 生产者未归属（BCI 0/10），由 2.4 在现有资源初始化证据内精确闭合，不新建全局 Region 机制。
4. **呈现为一条头。** 源代码是一条 `try (a; b)`，恢复文本同形；每层的 close 证据仍逐层验证（`close_of_level`/`close_handler`/`Suppressed` 原样），正常路径逆序 close 的既有证明复用。资源名沿用 `names` 拼写。
5. **与 catch 组合沿用 enclosing_clauses。** TWR+catch 的行几何（真 TWR 行 + 被包围用户行）在本几何上重验；冲突时拒绝，不放宽。
6. **证据。** 正例（2 资源 + 返回值 + 内层抛异常 + 外层 close 抛异常 + 正常关闭）、负例（行范围或同目标保护行被等宽 patch 错一级、返回尾部加效果 → 拒绝保持）、旧 `Guarded.two/three` 单行形状、三方对照（jadx 的展开输出记为其偏离）、重编译执行对照。

## Risks / Trade-offs

- 不同 javac 版本的单行/分段保护布局 → 两种均逐行证明，旧 `Guarded.two/three` 与新受控 fixture 同跑；未满足精确保护条件的其它编译器布局拒绝。
- 层级链吸收外层用户 catch 的旧风险 → `closes_something` 过滤与 unexplained-row 检查保持。

## Migration Plan

无迁移；golden/语料计数若变重录。

## Open Questions

无。
