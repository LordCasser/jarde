# 手写 Iterator 实现类 + JDK BiConsumer 位巡查（2026-10-05 root，负结果）

## 探针

[fixture/IT.java](fixture/IT.java)（`--release 8`）：own 类 `implements Iterator<Integer>, Iterable<Integer>`（cur/end 状态字段跨 hasNext/next 协作 + remove 抛 + iterator 返 this）、for-each 消费手写迭代器、`map.forEach((k,v) -> sb.append...)`（JDK BiConsumer 消费位 lambda）。

## 结果：**健康（已知族内），无新缺口**

- **sumRange 恢复**：`new IntRange(...).iterator()` + while hasNext/next 循环 + `(Integer) next()` 拆箱 cast——消费手写迭代器完整；
- **biConsume 恢复**：JDK `Map.forEach(BiConsumer)` 位 lambda 以 companion 形内联（`IT.lambda$biConsume$0$jarde(local1, p0, p1)` 重命名调用——既有 companion 机制），cast 链如实；
- **IT$IntRange**：ctor/hasNext/remove（throw UOE）/iterator（return this）恢复；**`next()Ljava/lang/Integer;` 整方法拒**（`return cur++;` 返回位——#81 已知 return x++ 族，explanation only=安全）；
- **桥方法呈现数据点**：伴生同时渲染真实 `next()`（拒，空体）与桥 `next()Object`（调 this.next()）——拼接时同名双签名编译失败=安全；桥折叠归已知桥呈现域（c1e5248e 数据点同源）；
- 行为（手工隔离替换 IntRange 为正确实现验证主体）`10/a=1;b=2;` 一致。

## 处置

负结果归档，不立 spec。
