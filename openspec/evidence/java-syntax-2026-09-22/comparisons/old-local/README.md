# 比较操作数的旧局部值

`patch.py` 对冻结 NumericComparisons.class 中唯一的 long_eq Code 精确插入 lconst_0/lstore_0：最初 lload_0 的旧值仍在栈上，slot 0 已改为 0，随后才读右操作数并执行 lcmp/ifne。没有新增永久 class；长度和 StackMapTable 同步调整。

`java -Xverify:all` 对 (5,5)、(5,0)、(0,0) 返回 7/9/7。2026-09-23 数值比较实现后的 debug CLI 保留 `arg0 = 0L`，并引用 BCI 0、8、11、5、4；原因明确指出 BCI 0 读出的旧局部值在最终 BCI 5 求值位置已不能用槽名表示。没有生成使用新 arg0 的错误比较。

这证明现有最终求值位置检查可覆盖该边界，无需新增缓存旧值、合成局部或比较专用别名机制。源码未完整恢复，因此不宣称该 patched 整类已重编译。
