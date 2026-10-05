# 重载方法引用决议面巡查（2026-10-06 root，负结果）

## 探针

[fixture/OM.java](fixture/OM.java)（`--release 8`）：`String::valueOf`（`Function<Integer,String>`——装箱后重载族）、`Integer::valueOf`（String/int 双形）、自建三重重载族 `OM::o`（int/String/Object）作为 SAM 目标、lambda 直传对照。

## 结果：**健康，无新缺口**（quotes=0）

- 方法引用经 invokedynamic 规范为**带显式装箱/cast 的 lambda 包装**（既有方法引用域呈现选择，#methodref 巡查结论外推到重载面成立）；
- **重载决议保行为**：`String::valueOf` 呈现 `String.valueOf((Object)(Integer)p0)`（选 Object 重载——与 javac 原决议 int 形不同名但行为恒等）；`OM::o` 呈现 `OM.o((Object)(Integer)p0)`（选 Object——与原决议一致）；`Integer::valueOf` 双形各自正确（String/int）；
- 4 个 SAM 参数方法的 `ordinary_generic_source_unproved` 签名投影拒 = **既有签名域**（2026-09-24 ordinary-parameterized-signatures 已登记），非新缺口；
- 剥离往返编译 exit 0、`-Xverify:all` 行为 `7/42/9/o3/L3` 逐行 IDENTICAL（含 lambda 直传对照）。

## 处置

负结果归档，不立 spec。方法引用×重载决议面确认覆盖。
