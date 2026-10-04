# 调用族巡查（2026-10-05 root，负结果）

## 探针

[fixture/IN.java](fixture/IN.java)（`--release 8`）：虚分派（调用点 Base、运行 Sub，含 `asBase()` 上转型返回）、invokeinterface 两条常见链（ArrayList 可变实现、Arrays.asList 不可变实现）、invokestatic、default 方法被匿名类消费（`d()+1`）。

## 结果：**健康，无缺口**（宿主 + 全部 4 个伴生类 quotes=0）

- 虚分派：`arg0.f()` 呈现于参数类型 Base 上（调用点静态类型正确）；`main` 中 `(IN$Base) local1.asBase()` 保留上转型 cast——忠实；
- invokeinterface：`ArrayList`/`List` 两条链均如实（raw 呈现 + `add((Object) "x")` 擦除 cast）；
- default 方法消费：匿名类 `IN$1` 呈现 `return this.d() + 1;`（default 继承调用如实）；
- 拼接五类型（含 `IN$I` 接口）→ `javac` exit 0 → `java -Xverify:all` 输出 `2/1/7/2/6` 与原 class **逐行一致**。

## 处置

负结果归档，不立 spec。调用族（virtual/interface/static/default-consumption）确认覆盖。
