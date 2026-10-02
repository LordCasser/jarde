# 静态初始化块尾随 return 巡查（2026-10-02）——JLS §8.7 违例

[super-default 巡查](../super-default-patrol/README.md) 附带发现（主线 `cc272d0f`）。证据复用 [super-default-patrol/results/F2.jarde.java](../super-default-patrol/results/F2.jarde.java)（fixture F2.java/F2.class 同目录）。

## 表现

F2 的 `<clinit>` 呈现：

```java
static {  // 呈现为类内静态块语句
    if (Boolean.getBoolean("boom")) { throw … }
    else { F2.value = 5; F2.other = init(); return; }   // ← JLS §8.7 禁止
}
```

javac 拒编（"return Outside method"）→ 含静态块的类**整类不可重编**。判别：构造器/普通方法中尾随 `return;` 合法且为既有忠实呈现（不受影响）；唯静态/实例初始化块语境非法。

## 根因假设

`<clinit>` 体呈现复用普通 void 方法语句发射（末尾 return 显式发射）；初始化块语境无对应豁免。

## 处置方向

`recover-init-block-return-suppression`（窄呈现切片）：静态初始化块（`<clinit>`）与实例初始化块语境的语句发射抑制尾随（及块内冗余）`return;`——只影响初始化块呈现通道，方法/构造器零变化。F2 整类可重编；既有 `<clinit>` 家族（enum `<clinit>` 折叠等）diff 逐字核对。

原 class 为行为基准。
