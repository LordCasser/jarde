# CF-12 主线独立验收

root 在 CF-12 直接 `return` 投影补丁和 CF-13 实现合并后的主线重新构建 CLI，SHA-256 为 `9518aa8e0d5e34ab6d5ce71b6222db5a38b009ebea6467a228553d3db44cc4f9`。独立运行固定 [replay-after.py](replay-after.py)：原 class、固定 JADX 与 Jarde 的完整类源码均以 `javac --release 8 -g:none` 重编，并以 `java -Xverify:all` 运行 11 行逐字一致。Jarde 源码 SHA-256 为 `c9f39eae46e54b086fc4a2b5d57ef5b5777a5dd547bc1725d96386d65649f4f5`，与实现者归档相同；`case LOW:`、`return HIGH;` 只出现在有完整字段表的类源码中，单方法恢复仍输出数值。两个派生名称分别锚定物理字段及 BCI 1/20。

root 另外在合并态运行 `integer_constant_name_tests`，5/5 通过，其中新增的 `ReturnOnly` 用例证明 `return HIGH;` 的投影不依赖同一方法含可投影 `case` 标签。无完整字段表、重复值、参数遮蔽、非整数字段及非法名字等边界由同组拒绝测试约束。固定 JADX 测试本身没有覆盖所有整数 switch lowering，因此本结论只关闭常量名质量差距，不宣称 CF-12 全单元追平。
