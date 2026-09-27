## 1. 捕获来源闭合

- [ ] 1.1 在既有物理捕获证明旁加入准确 synthetic-final `D` 字段、唯一 `(D)V` 构造器和完整 SSA 写入/读取链验证；用正例及错误字段 owner/descriptor、第二写入、异常范围、method-handle 使用负例验证。
- [ ] 1.2 证明根方法唯一直接返回分配的实参来自未改写的根 `double` 参数槽，且 child 字段消费闭合；用错槽、槽复用、额外分配、跨类引用负例验证拒绝原因。

## 2. 原子匿名投影

- [ ] 2.1 在同次 child AST 中按准确 BCI/字段身份将每次捕获字段读取改写为根参数名，并沿现有接口匿名体 emitter 一次性提交；用完整源码断言 `new Runnable() { ... }`、没有源级 `Capture$1` 引用及物理 child 独立查询验证。
- [ ] 2.2 为任一读取不匹配、预算不足和取消补全原子拒绝测试，确认根源码及 child 物理报告都没有半份改写。

## 3. 三方验收

- [ ] 3.1 对固定 DT-08 输入及修后 Jarde 完整源码分别运行 `javac --release 8`、`java -Xverify:all`，与原 class 和固定 JADX 比较普通值、负零值、效果顺序并保存可重放哈希；注明原 Jarde 编译失败基线。
- [ ] 3.2 回归原有无捕获匿名接口及 `this$0` 捕获切片，运行定向测试、workspace check、`cargo fmt --check` 与 OpenSpec strict，记录尚未覆盖的多字段/外层实例组合而不扩大本改动。
