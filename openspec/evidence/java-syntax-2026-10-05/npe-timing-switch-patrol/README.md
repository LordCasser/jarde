# switch(null) NPE 时机巡查（2026-10-05 root，负结果——经典 NPE 时机陷阱全过）

## 探针

[fixture/NS.java](fixture/NS.java)（`--release 8`）：String switch 对 null（NPE at hashCode）、enum switch 对 null（NPE at ordinal）、装箱 Integer switch 对 null（NPE at intValue）、`"a".equals(k)` 字面量先行 if 对照（null 安全）。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）——NPE 触点保留

- **String switch 折回 `switch (arg0)`**：重编后 Java switch-on-string 的 hashCode 语义使 null 在同点抛 NPE；
- **enum switch 呈现 `switch (arg0.ordinal())`**：NPE 触点（ordinal 调用）显式保留；
- **装箱 switch 呈现 `switch (arg0.intValue())`**：拆箱调用显式保留（与装箱 switch 巡查一致）；
- 字面量先行 if（`"a".equals((Object) k)`）对照 null 安全恢复；
- 行为 `d/NPE-str/NPE-enum/NPE-boxed` 逐行 IDENTICAL——**三形 NPE 时机精确**（错误重排会改变异常类型或丢失）。

## 处置

负结果归档，不立 spec。NPE 时机族（switch 三形 + if 对照）确认覆盖。
