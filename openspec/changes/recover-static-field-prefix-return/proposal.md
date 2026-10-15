# Proposal

## Why

当前产品 ba3fa76af 的两份原始 FO（javac23 --release 8 与真实 javac8）均能在 Java8/23 输出 `1/100`，但最终CLI的完整FO源码均因 `bump()I` 缺失return而重编失败。实因是本类静态int字段的前置递增返回新值尚未恢复；`evalThenFin()` 已正确保存返回值后执行finally，不能把辅助方法缺口当finally问题。

## What Changes

- 对本类、非final/nonvolatile静态int字段，在完整无handler单块内证明一次读、加一、复制新值、写回并直接返回同一新值，恢复可读的前置更新返回；保留字段身份、求值次数、int溢出和来源。
- 新值只由该次加法产生，不能写回后再读字段来取得返回值；证据不闭合时沿既有拒绝和停止契约处理。
- 以两编译腿的完整FO输出原样重编、固定main和新增独立driver的非零/溢出/return-before-finally对照验收；现有实例字段前后缀、静态postfix旧值及无值更新作为兼容控制。

前置条件：先完成 preserve-reference-overload-dependency-proof-stops 的自身CI、归档与源码绑定，再应用本计划。当前只立项，未实现或重建运行成功。基线与实际失败保存在 results/repro-v1；原始运行控制通过不代表Jarde输出通过。

非目标：不改finally、通用dup求解、其它字段更新消费者、全局预算/类型系统；不在本片扩展减一、非int/volatile/final、跨类字段、任意表达式位置或跨块/handler形态；不重写旧实例更新表示，不合并DN/CD过时计划，不增加全局语法覆盖数字。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：新增已证明的静态int前置递增直接返回，约束新值身份、一次读写、来源、拒绝/停止与完整源码行为验收。

## Impact

预计复用 jarde-java 的字段声明/field@1、SSA/effects、预算证明、Return与来源发射接缝，必要时在现有AST/emitter增加最小的前置更新表达。保留库优先分层和薄JSON CLI，无新依赖/API、通用更新框架或宿主运行时依赖。已有实例DUP_X1八指令规则、void-only单位更新、旧值snapshot均不覆盖本次静态DUP新值，不能通过放松其原判据代替本片证明。
