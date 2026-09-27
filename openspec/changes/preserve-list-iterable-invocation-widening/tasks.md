## 1. 证明并接入平台上溯

- [ ] 1.1 在现有调用实参类型判定中加入 Java 8 精确 `java.util.List → java.lang.Iterable` 安全关系；用同类型、`Object`、数组、已证明重载和未证明用户类型控制检查准入边界、原子拒绝与预算/取消路径。
- [ ] 1.2 确保生成的目标参数表达式只包住原实参一次并保留 producer、调用 BCI；用 source-map/origin 断言及 `ListToIterable` 实参求值计数验证。

## 2. 完整类验收

- [ ] 2.1 将独立 `ListToIterable` 输入加入定向 fixture；以 `javac --release 8` 重编原/JADX/Jarde 完整类，再以 `java -Xverify:all` 运行；三方逐行输出均为 `called`，生成类的消费调用恰执行一次。
- [ ] 2.2 复跑调用参数、已证明引用重载、数组上溯与直接/平台 Iterable 的相邻验收；确保其它未证明引用关系仍拒绝，并记录源码、class 与产物 SHA-256。
