# 字段访问交互巡查（2026-10-05 root，已立项域的实证锚）

## 探针

[fixture/FW.java](fixture/FW.java)（`--release 8`，javac 23 腿）：内部类**读 final**（ctor 定值，无写访问器）、**读写混合**（`mut = v; return mut` → 一写一读两访问器）、**显式 `FW.this.fin` 限定读**、实例内部类写外类**静态**字段（无访问器，直接 putstatic 合法）。

## 结果

- 读形（final `access$000`、普通 `access$100`）**恢复**；静态写**无访问器**（javac 对静态字段不发访问器——`FW.statW = 0` 在 clinit、`SW.w()` 直接 putstatic，呈现 `FW.statW = 9` 走内部类渲染）；
- **写形 `access$102` 被拒**（"the instruction at BCI 2 is not part of the provable subset"）——与真 8 CA 探针、WA 8/9 基线**同形同因**，正是 [recover-write-accessor-field-types](../../../changes/recover-write-accessor-field-types/)（EM-15，在飞）的目标；其 fixed 渲染已 9/9 恢复；
- `main` 的 12 条引注是写访问器拒绝的**级联**（`f.new W()`/`f.new SW()` 两个成员构造站点 + 读写混合调用链），同 CA 探针的级联机制——EM-15 落地后应全部解锁。

## 处置

不新立（EM-15 在飞域）；本探针作为 EM-15 的**第四实证锚**归档（WA 9 型、CA 真 8、CB 上下文对照、FW 交互级联），供验收时复核交互面（读 final/读写混合/静态写）是否随写形解锁。
