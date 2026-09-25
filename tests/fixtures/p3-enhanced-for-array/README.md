# 数组增强 for 的准入反例

`ArrayForeachRefusal.java` 用 `javac --release 8 -g:none -Xlint:-options -d v8 ArrayForeachRefusal.java` 编译。冻结的 `v8/ArrayForeachRefusal.class` SHA-256 为 `19a4f569b98fae469cddc625ab2b11af3d2a4dae5635b4526e56b0c007f4e639`。

这组方法与正例一样先捕获数组和长度，形成可恢复的计数 `for`。体内索引额外用途、长度和读取数组不一致、元素局部在循环后逃逸仍是拒绝例；`effectInBinding` 的元素读取先于 `tick()`，在 wrapped-array 子切片中成为安全正例。`indexAfterLoop` 另测归纳索引终值的外部读取。`ArrayForeachRefusalRunner.java` 用于原 class 与 Jarde 完整类重编后的 `-Xverify:all` 执行对照。

`ArrayForeachTransfers.java` 也用同一 `javac` 命令编译，class SHA-256 为 `51fb9055101b7b130f8e402334ee59056d32d36e18fdef95da6078cf1a98f622`。其 `continue` 路径可投影；`break` 路径目前不形成 `ForHeader`，保留普通 `while`。对应 runner 同样执行重编对照。
