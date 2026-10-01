# 混合形态：构造实参 + 常量专属体（2026-10-01 巡查）

[family 巡查](../README.md)的剩余组合形态取证（主线 `02177997`）。固定 fixture：`p.Combo`（[fam.jar](fam.jar)，SHA 见 [sha256.txt](sha256.txt)；源含 `ADD(1){…} MUL(2){…} ID(0)` 三常量——两带体带参、一普通带参），行为基线 [orig.out](orig.out)（`7`/`12`/`0`）。

## 结果

主线输出逐字段降级（[combo.base.java](combo.base.java)：`public static final p.Combo ADD;` ×3、`valueOf` 拒绝）。判别事实：**顶层名**（排除嵌套名因素）、**常量体与构造实参两能力各自已闭合**（constant-bodies 义务通道、arbitrary-arguments grammar），失败在二者的组合——带体常量经匿名子类构造（`new Combo$1(String,int,int)`，int 参随委托链转发），常量体折叠的义务证明此前只覆盖零源参形态（子类 ctor 恰 `(String,int)` + 桥 `(String,int,$1)`），组合形态的子类 ctor 为 `(String,int,<用户参>)`、桥为 `(String,int,<用户参>,$1)`。

## 处置

`recover-enum-mixed-constant-bodies`：常量体义务证明参数化——子类/桥 ctor descriptor 接受任意实参 grammar（arbitrary-arguments 已建立的参数集），委托链逐参转发证明；常量呈现 `NAME(<args>) { <body> }`。普通带参常量与纯带体常量（两已闭合能力）逐字不变。属两个既有证书的交叠组合，无新机制。

原 class 为行为基准。
