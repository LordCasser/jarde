# task 2.2 Java 控制源草稿 v1

本目录只是待 root Cargo 阶段结束后编译检查的源草稿和 runner。准备阶段没有运行 Java、Javac、Javap、Cargo、Git 或 rustfmt；也没有在这里生成任何 class、命令流或宣称编译状态的 manifest。执行脚本后，每个 JDK 会把 `NestedControls` 与 `BoundaryControls` 编译到独立输出目录，避免负例与嵌套正例互相影响，并在 `run-NNN/manifest.json` 记录实际命令、工具和源/class hash、退出码、双流及 javap BCI。

`NestedControls.nested()` 是独立的完整正例，数组元素为 `new StringBuilder(new StringBuilder(mark("nested")))`；observer 只取第 0 项赋给 `Object` 并调用 `println(Object)`。`BoundaryControls` 没有 main，保留后半元素 `Long` 结构路径、旧数组写、fresh array 重复/逆序实际存储、显式 `if` 跨 block，以及合法但落在闭合 Number 证明表外的 `BigDecimal` 类型边界方法。`BigDecimal` 到 `Number` 是合法 Java 赋值；若实现将来扩充了该 closed fact set，此控制的保守拒绝预期会失效，不能把它解释成非法 Java。

两类源分开编译、只采集 Javac/Javap 事实。脚本不调用 `java`，本草稿不是运行时语义证据。root 的审查决定后再将适用边界迁移到独立测试。
