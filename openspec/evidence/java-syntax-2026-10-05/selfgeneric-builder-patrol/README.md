# 自引用泛型 builder（unchecked 自类型 cast）巡查（2026-10-05 root，负结果）

## 探针

[fixture/SR.java](fixture/SR.java)（`--release 8`）：`Builder<T extends Builder<T>>` 自引用泛型界 + `(T) this` unchecked 自类型 cast（Retrofit/OkHttp 风格 fluent API 骨干）、抽象 `self()`、具体叶子 HttpBuilder 继承+自身返回链、`use()` 混合返回类型全链（named→at→secure→build）。

## 结果：**健康，无缺口**（三伴生 quotes=0）

- `use()` 全链恢复：`((SR$HttpBuilder) ((SR$HttpBuilder) new HttpBuilder().named("api")).at(3)).secure(false).build()`——**unchecked cast 消费位（checkcast 到叶子类型）如实呈现**，混合返回类型（T 转具类）正确；
- `SR$Builder` 抽象类：字段/抽象 `self()` 声明（无 Code 诚实注记）/`named` 的 `(T) this` unchecked cast 体恢复；
- `SR$HttpBuilder`：继承+实例字段+方法链恢复；两个 class Signature 投影拒（`class_generic_source_unproved`/类型变量 scope）=**既有 Signature raw 退化域**（类头泛型不呈现），非新缺口（与 #49 RG IntNode 同域）；
- 行为 `api:3:false` 一致（渲染原样编译运行）。

## 处置

负结果归档，不立 spec。自引用泛型域（自类型 cast/抽象泛型方法/叶子继承链）确认覆盖。
