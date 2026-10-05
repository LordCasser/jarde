# null 安全链式 setter（框架 builder 模式）巡查（2026-10-05 root，负结果）

## 探针

[fixture/LG.java](fixture/LG.java)（`--release 8`）：null 安全三元 setter（`this.name = n != null ? n : "default"`）、守卫 throw setter（`if(s < 0) throw`）、`Collections.emptyList()` 工厂默认、varargs `Arrays.asList` 实参、完整链 `new LG().name(null).size(3).tags(null).build()`。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- null 安全三元/守卫 throw/工厂默认/varargs 实参全部恢复；main 双链（含 `(String) null` 显式转型实参与 saved-printStream 变量提取形）如实；
- 行为 `default:3:0`、`x:1:1` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。真实框架 builder 模式（Lombok/手写链式 setter + null 守卫）确认覆盖。
