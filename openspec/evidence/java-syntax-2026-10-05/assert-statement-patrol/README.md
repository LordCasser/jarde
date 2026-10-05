# assert 语句巡查（2026-10-05 root，负结果——经典难点全过）

## 探针

[fixture/AS.java](fixture/AS.java)（`--release 8`）：assert 带 message（`assert x>0 : "pos expected"`）、assert 裸形、`desiredAssertionStatus()` 查询；[fixture/ASX.java](fixture/ASX.java)：触发路径（负实参 + `-ea`）。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- **assert 双形完整还原**：javac 的 `$assertionsDisabled` 合成域 + `getdesiredAssertionStatus` clinit 舞蹈 + if/goto 断言路径被还原为 **`assert` 语句**（带 message 形 `assert arg0 > 0 : "pos expected";` 与裸形）——众多反编译器只还原为裸 if 形，jarde 直接还原为语句级；
- `desiredAssertionStatus()` 查询如实；
- **双腿语义验证**：渲染源 `javac` exit 0（javac 为 assert 重新生成合成域——字段级无损）；常规运行 `3/false` 一致；`-ea` 启用路径 `3/true` 一致；**触发路径**（`-ea` + 负实参）两腿同抛 `java.lang.AssertionError`（ASX 带消息形 `AssertionError: pos`、AS 裸形）——断言语义双向保真。

## 处置

负结果归档，不立 spec。assert 族（带/裸 message、启用/禁用/触发三路径）确认覆盖。
