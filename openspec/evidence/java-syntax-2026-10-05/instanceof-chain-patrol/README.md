# instanceof 链类型分派巡查（2026-10-05 root，负结果——多态替代模式全过）

## 探针

[fixture/IT.java](fixture/IT.java)（`--release 8`）：**instanceof 链类型分派**（子类先测——顺序语义关键：C→B→A）、cast 消费（`((C) x).id()`）、Object 位负形测试、instanceof 后无 cast 直接调用（guard 形）。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- instanceof 呈现 `(Object) arg0 instanceof IT$C`（**先上转型再测**——javac 物理序列如实；语义等价）；链序精确（子类先测，A 兜底分支）；cast 消费 `((IT$C) arg0).id()` 如实；guard 形（instanceof 后直接 `x.id()`——A 自有方法无需 cast）正确；Object 位负形测试（false/true 分野）；
- 四类型拼接（宿主+A/B/C 伴生）`javac` exit 0、行为 `c:C/b:1/a:A/false/true/guarded:B` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。instanceof 分派族（链序/cast 消费/负形/guard）确认覆盖。
