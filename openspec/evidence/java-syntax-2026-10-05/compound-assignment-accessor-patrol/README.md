# 复合赋值访问器巡查（2026-10-05 root）

## 结论（一段话）

内部类对外类私有字段的**复合赋值**（`seed += 1`）生成**读-改-写形访问器** `access$012(LCA;I)I`（`aload_0; dup; getfield; iload_1; iadd; dup_x1; putfield; ireturn`，7 指令）——该形被拒（`not recovered`），且**与上下文无关**（构造器与普通方法两探针同拒）；纯读形 `access$000` 与（EM-15 片正在实现的）纯写形均不受影响。`main` 中 `c.new Nut()` 的引注是该拒绝的**级联**（构造器体不可证 → 构造站点拒），非独立缺口。

## 判别实验

| 探针 | 复合访问器上下文 | 源码区 | 纯读 `access$000` |
| --- | --- | --- | --- |
| [fixture/CA.java](fixture/CA.java) | 构造器 `Nut(){ seed += 1; }` | quotes=10 / notrec=1 | **恢复** `return arg0.seed;` |
| [fixture/CB.java](fixture/CB.java) | 普通方法 `void bump(){ seed += 1; }` | quotes=4 / notrec=1 | — |

两上下文同拒 → 判别变量是**形状**（7 指令复合形），非构造器上下文。jxr/javap 原始记录见 [results/](results/)。

## 对既有登记的更正

allocation-qualifier 片（`38b50d19`）实现者登记的"子类构造器内 `access$000` 被 member-table 判据拒绝"——root 干净探针复验：纯读形**恢复正常**，其初版 Pod 探针观察到的 member-table 拒绝是复合形连带前的中间态/或其特定族的 member-fold 交互。**准确缺口 = 复合赋值访问器形状**，root 已同步更正 summary.md 的登记行。

## 与 EM-15 的关系

EM-15（`recover-write-accessor-field-types`，在飞）的 Non-Goals 明确排除复合写形："`i += 1` 生成的读-改-写访问器——那有不同的 opcode 结构，须另行取证"。本巡查即该取证：形状已定型（前置 `dup; getfield` + 运算 + 后半与纯写形同构的 `dup_x1; putfield; xreturn`）。**颗粒度：窄到中**——与 EM-15 同族（描述符封闭表 + 结构判据），但 opcode 序列更长且含运算段（每运算符一族：`+ - * / % & | ^ << >> >>>`，及值类型），宜在 EM-15 落地后作为姊妹片复用其表。**暂不立项**（等 EM-15 落地，避免同函数双开）。

## 处置

取证归档；登记到 summary.md 独立缺口行（已更正原 ctor-access$ 登记）；EM-15 落地后按姊妹片立项。
