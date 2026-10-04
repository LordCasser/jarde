# 方法引用边角巡查（2026-10-05 root，负结果+呈现注记）

## 探针

[fixture/MR.java](fixture/MR.java)（`--release 8`）：`super::name`（super 接收者实例方法引用）、`Kid::new`（类构造引用，Supplier 形）、`int[]::new`（数组构造引用）。

## 结果：**健康（三形全部恢复）**（宿主+伴生 quotes=0）

- **`super::name`**：`boundSuper()` 呈现为 `return () -> this.lambda$boundSuper$0();` + 物理 helper `lambda$boundSuper$0() { return super.name(); }`——**lambda 包装 + companion 显式呈现**（companion 未内联进 `boundSuper` 是因 `this.lambda$…()` 转发链，语义精确：helper 内 `super.name()` 调用如实）；
- **`Kid::new` 构造引用**：`return MR$Kid::new;` **直接方法引用呈现**（invokedynamic 的构造引用如实还原）；
- **`int[]::new` 数组构造引用**：`arrRef` 静态字段初始化（在 clinit）呈现为 `(java.lang.Object arg0) -> new int[(java.lang.Integer) arg0];`——**lambda 形**（装箱参数 + 数组构造调用），语义等价（原 `int[]::new` 是 Function<Integer,int[]>；装箱 cast 呈现忠实）；
- 拼接三类型 → `javac` exit 0 → `b/b/3` 与原 class **逐行一致**。

## 呈现注记（非缺口）

- `boundSuper` 的 Signature 投影拒绝注释（raw `Supplier`）属已登记的 Signature 域；`arrRef` 的 `field_generic_body_unproved` 同（静态泛型字段——其初始化呈现在 clinit 内**正确**（lambda 形可编译），与 [generic-static-field-init-patrol](../generic-static-field-init-patrol/README.md) 的损坏文本形不同：该形初始化是 invokedynamic（lambda），走了 lambda 内联路径而非损坏的字段内联路径——**同一"投影拒绝"下两条呈现路径，一条好一条坏**，进一步佐证损坏片的落点在字段内联分支而非投影拒绝本身）。

## 处置

负结果归档，不立 spec；`arrRef` 数据点并入损坏片的落点证据（clinit+lambda 路径好 / 字段声明内联路径坏）。
