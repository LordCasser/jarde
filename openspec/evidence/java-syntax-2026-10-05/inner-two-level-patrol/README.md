# 双层内部类 this$0 链巡查（2026-10-05 root，负结果）

## 探针

[fixture/DC.java](fixture/DC.java)（`--release 8`）：`DC → Mid（内部）→ Leaf implements Runnable（Mid 内嵌第二层）`；Leaf.run() 内**双层逃逸**（`Mid.this.depth` + `DC.this.tag`）——GUI 适配器真实形。

## 结果：**健康，无缺口**（三伴生 quotes=0）

- 双层逃逸引用以物理事实呈现：`this.this$1.this$0`（Leaf→Mid→DC 两跳 synthetic field 链）；
- 跨层私有访问经桥访问器：`DC.access$000(this.this$1.this$0)`（读 outer 私有字段）+ `DC$Mid.access$100(this.this$1)`（读 Mid 私有字段）——与 #56 access 家族一致；
- 拼接三伴生编译 exit 0、`-Xverify:all` 行为 `outer/1/outer` + `done:outer` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。内部类域第二层深度确认覆盖（this$1.this$0 链 + 双层桥访问器）。
