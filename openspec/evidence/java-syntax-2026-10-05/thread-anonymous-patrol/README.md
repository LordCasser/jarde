# Thread/Runnable 匿名类 + volatile 巡查（2026-10-05 root）

## 结果：主体健康；capture-ctor-super-order 债务新锚（Thread 形）

- **宿主全恢复**（quotes=0）：volatile stop 字段（flags 呈现）、`new TH$1(this)`/`new TH$2(this)` 伴生实例化、joinAll（throws InterruptedException + for-each join）；
- **TH$2（匿名 Runnable，带实例字段 local）全恢复**：`super(); this.this$0 = arg1; this.local = 0;` ——**合法重排**；
- **TH$1（匿名 Thread 子类，无后续字段初始化）渲染非法序**：`this.this$0 = arg1; super();` —— javac exit 1（super 须首句）。javap 证实**两伴生 ctor 字节码逐条相同**（putfield this$0 在 invokespecial super 前）——差异在渲染层：有后续初始化的伴生被重排、无后续初始化的保持原始序。**安全方向**（编译失败=第一不变量不违反），归 [capture-ctor-super-order 片](../../../changes/recover-capture-ctor-super-order/)（double-brace 巡查 c0b77f9f 所立）**新锚：Thread 匿名形（无尾随初始化的重排缺口）+ 同形异序不一致**；
- run 体（while !stop / ticks+=1 / break / println 链）双伴生恢复；行为 `task:1/task:2/101` 一致（原版）。

## 处置

capture-ctor-super-order 片补 Thread 匿名锚（判别点：伴生 ctor 是否有尾随字段初始化）；其余不立项。
