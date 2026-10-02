## Context

[巡查证据](../../evidence/java-syntax-2026-10-03/inner-class-folding-patrol/README.md)：N1 fam.jar（`new N1$Inner(this, arg1)`、`access$000` 桥、`outer.new` 无证明）。既有机械：两片折叠（`StaticMembers`/`StaticMembersWithInstance` + 投影四件套）、synthetic-ctor-super-order（ctor 合成序知识）、capture-ctor 家族分离呈现（`C1$1` 惯例——**分离呈现不动**）。**第一个取证义务（三项）**：(a) javap N1 家族——`Inner` ctor 首参与 `this$0` 存储序、`access$000` 桥签名族、`Stat.use` 的限定 new 降低（`requireNonNull` 舞蹈或 getfield 直通）与 `new N1().new Inner(3)` 的静态语境降低；(b) 读 mixed 片 `StaticMembersWithInstance` 的装配缝——非静态候选走窄通道的选择点如何改为"折叠+隐藏合成物"；(c) 限定 new 恢复缺口的真实拒绝码（决定它是否须先行独立切片——若舞蹈恢复是前置，则本片先交付 this/直接 new 形，限定形登记）。

## Goals / Non-Goals

**Goals:** 非静态子折叠 + this$0 消隐 + 非限定/限定 new 语法呈现 + access 桥消隐与调用位重写；N1 家族集行为一致。**Non-Goals:** 匿名类内联（既有分离惯例）；局部类；接口默认方法的捕获；`access$NNN` 写桥的方法调用形（`access$100(o, v)` setter 形——验一形登记）；孙代；非捕获字段直接访问（本就直通）。

## Decisions

1. **折叠与消隐分层落地**：折叠机械复用（非静态行准入 + 装配合并）；消隐是呈现层语境规则——折叠作用域内 `this$0` 位 ctor 消参、字段隐藏、`access$NNN`（ACC_SYNTHETIC 静态 + 名模式 + 桥体单读/单写直通证明）隐藏；分离呈现完全不动（语境=折叠文本）。
2. **构造限定形**：限定值 SSA=外围 this → `new Inner(args)`；其它限定值 → `qualifier.new Inner(args)`（限定 new 恢复若受阻于舞蹈证明，按取证 (c) 决定登记或先行窄片——如实报告）。
3. **桥调用位重写**：`access$000(o)` → `o.base`（读形 MVP）；setter 形验一形登记。重写经折叠域 token 机械（同族拼写重写通道）。
4. **验收锚定**：N1（`10/7/13`，家族集含分离 Stat 語境）+ 变体（捕获多字段、桥写形、深一层 `A.new B.new C` 登记）；负例（分离呈现逐字不变、纯静态族零回退）。

## Risks / Trade-offs

- **限定 new 舞蹈证明是前置** → 取证 (c) 先行；若前置，本片交付 this/直 new 形 + 登记，禁止为凑验收放宽舞蹈证明。
- **消隐破坏行为等价**（this$0 有空检查副作用）→ 消隐只改呈现，重编由 javac 重新降低（重降序与原字节码等价即行为一致）；三方运行对照是门禁。
- **桥体非直通**（多读/条件）→ 桥不隐藏，保守呈现。
