# 参数校验连环 + 异常消息构造巡查（2026-10-05 root，负结果）

## 探针

[fixture/PC.java](fixture/PC.java)（`--release 8`）：Commons Validate 风格**五连环 if-throw**（null/isEmpty/范围/范围带名/null NPE——无 else 源形）+ SB 容量构造器消息工厂（`length()+16` + `append('=')` char 位）+ main try-catch 消息消费链。

## 结果：**健康，无缺口**（全类 quotes=0 含 main）

- 连环 if-throw 折成 **else-if 链**（控制流等价规范化——各分支全 throw，无穿透差）；五种异常消息形（字面量/SB 拼接/多段拼接 `+ age + " for " + name`）完整；
- `new StringBuilder(tpl.length() + 16)` 容量构造器 + `append('=')` **char 实参位**（cappend 精确）+ toString；
- main 的 try-catch 连环 + `(String) null` 特化实参 + getMessage 消费全恢复；
- 行为往返 `A:name is null / B:age: -1 / C:age too big: 200 for bob / D:extra required / fmt:count=42` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。校验连环域（守卫 throw 链/消息构造/容量 SB）确认覆盖。
