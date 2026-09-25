# 类级 CLASS-retention 注解与拒绝边界

`run_boundary_audit.py` 从 `tests/fixtures/class-annotation-uses/src/` 以 `javac 23.0.1 --release 8 -g:none` 重编译输入，记录 class SHA、物理属性跨度、原始注解条目及 `javap -v -c -p` 输出。重放命令：

```sh
python3 openspec/evidence/java-syntax-2026-09-22/class-annotation-uses/boundaries/run_boundary_audit.py
```

`HiddenTarget.class` 的 `RuntimeInvisibleAnnotations` 位于 class byte `[184, 201)`，内容 `[190, 201)`；内容含一条 `LHiddenTag;`，`value` 为 `I` 常量池索引 13。原 class 的反射输出为 `0` 和 `null`，证明它并非运行时可见。原 class SHA、class 长度、`javap` 证据与 runner 输出保存在 `generated/summary.json` 和旁边日志中。

脚本另外固定三项受控输入：复制同一 annotation_info 两次、将 element_value tag 改为未知的 `Q`、以及将等长常量池类型描述符改成不能作为 Java 名称的 `LHidden-xx;`。摘要记录每个变体 SHA、属性跨度和相对原 class 的字节改动。reader 应分别保留两个原始重复事实、拒绝损坏属性且不发布部分事实、保留不可拼写类型事实；源码装配保守拒绝重复类型及非法 Java 名称。属性预算拒绝由 `jarde-reader/tests/class_annotation_facts.rs` 覆盖。

`Tag`/`Tags`/`DuplicateTarget` 则由 Java 8 源码生成正常的 Repeatable 容器，供对照实际 javac 合法写法；类源码无法从这一可见的单个容器属性反向拆出两个原始 `@Tag` 声明，因此不会把它当成“重复属性条目”的证据。这里的重复项是单独构造的 classfile 属性内容。
