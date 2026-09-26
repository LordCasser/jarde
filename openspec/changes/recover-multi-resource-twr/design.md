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

## Goals / Non-Goals

**Goals:** 层级几何改为「每层行止于该层语句末」；一条 N 资源头的呈现；close 逆序与抑制链既有证明复用；两份冻结 fixture（release 8 无检查形状为主）。

**Non-Goals:** 单资源几何改动；ifnull 资源头新证明；TWR+多捕获；close 顺序放宽；把 TWR 失败降级为普通 try/catch。

## Decisions

1. **几何按嵌套语句读。** javac 的多资源 lowering 等价于语句嵌套：`try (r1) { try (r2) { body } }` 的行范围各自从自己的资源初始化前开始、在自己的正常 close 链起点结束。校验改为：chain[i] 的行 end = 层 i 的正常 close 链起点（即 chain[i+1] 行 end 与内层正常 close 完成点的关系不再使用）。备选「保持单几何、放宽等式为包含」会放进不等于该层语句的字节，拒绝。
2. **呈现为一条头。** 源代码是一条 `try (a; b)`，恢复文本同形；每层的 close 证据仍逐层验证（`close_of_level`/`close_handler`/`Suppressed` 原样），正常路径逆序 close 的既有证明复用。资源名沿用 `names` 拼写。
3. **与 catch 组合沿用 enclosing_clauses。** TWR+catch 的行几何（真 TWR 行 + 被包围用户行）在本几何上重验；冲突时拒绝，不放宽。
4. **证据。** 正例（2 资源 + 返回值 + 内层抛异常 + 外层 close 抛异常 + 正常关闭）、负例（行范围被手工 patch 错一级 → 拒绝保持）、三方对照（jadx 的展开输出记为其偏离）、重编译执行对照。

## Risks / Trade-offs

- javac 9+ 与 Eclipse 编译器的行几何差异 → fixture 冻结 javac 8/9 两种；Ecj 形状不在本 change 断言，遇到即拒绝保持现状。
- 层级链吸收外层用户 catch 的旧风险 → `closes_something` 过滤与 unexplained-row 检查保持。

## Migration Plan

无迁移；golden/语料计数若变重录。

## Open Questions

无。
