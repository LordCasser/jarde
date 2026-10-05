# 内部类跨类私有访问 + long 位掩码巡查（2026-10-05 root）

## 发现：`access$002` 跨类 int 写-返回赋值形 not recovered（EM-15 家族残留位点）

内部类写外部私有字段（`secret = v` in Probe）→ javac8 生成 `access$002(LIA;I)I`（**返回赋值结果的跨类写访问器**）——IA 端整方法 not recovered（"instruction at BCI 2 is not part of the provable subset"）。EM-15 已覆盖同类写访问器 9 类型（含 assignment_result 的 Z/J 描述符门 build.rs:22538）；**跨类 receiver 窗口形（首参 LIA;）为残留位点**——EM-15 spec 预期的"姊妹实例形路径隔离"确证。读取（access$000）与方法转发（access$100）跨类均恢复。

## 健康面（负结果）

- **Probe 端全域恢复**：`IA.access$000(this.this$0)` / `access$002(this.this$0, arg1)` / `access$100(this.this$0, arg1)` 合成桥调用+`this$0` 物理事实如实；
- **静态 long 位掩码完美**：`lflags |= 1L << b` → `IA.lflags = IA.lflags | 1L << arg0`（>32 位惯用法，静态复合边界锐化再证）；`(lflags & 1L << b) != 0L` 测试位；行为 `true/false/1099511627778`；
- roundTrip（宿主内 `new Probe()`+链式调用）恢复；行为 `56/50` 一致。

## root 自纠错（提取教训 +1）

python -c 双引号内 `access$000` 被 bash `$000` 展开污染 → 假 absent；raw grep（单引号）为准。与第 84 前沿提取 bug 同列：**结论前必须 raw grep/打印原始切片**。

## 处置

access$002 跨类形登记为 EM-15 残留位点（账本 access$ 家族行）；不另立片（EM-15 已合，残留归其家族后续窄片）；long 位掩码不立项。
