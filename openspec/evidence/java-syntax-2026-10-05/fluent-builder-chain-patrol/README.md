# 流式构建器链巡查（2026-10-05 root，负结果——真实代码最常见模式之一全过）

## 探针

[fixture/FB.java](fixture/FB.java)（`--release 8`）：**this 返回链方法**（builder 模式）、纯链（`new B().a(1).s("x").build()`）、拆语句形（对照）、短链、**半链半引用**（`B b = new B().a(4); return b.s("z");`——链结果具名后续用）。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- **纯链逐字还原**（`return new B().a(1).s("x").build();`——分配+三连调用一气呵成）；拆语句、短链、半链半引用（链前缀+具名后续）全部如实；
- this 返回方法体（`this.a = arg1; return this;`）与 build 内 StringBuilder 链恢复；
- 行为 `1:x/2:y/3:null/4:z` 逐行 IDENTICAL（含 reuse 半链的 null 段——求值序精确）。

## 处置

负结果归档，不立 spec。流式链族（纯/拆/短/半具名）确认覆盖——真实业务代码（builder/gson/okhttp 风格）的核心模式健康。
