# Root focused evidence

v1 的144项功能回归通过，但P5计费3通过/2失败/1忽略，整体failed与原始流完整保留；随后原ignored recorder真实执行1项，核六行仅一行变化。只改六个精确费用常量及说明后，v2重新执行受影响P5（5通过/0失败/1忽略）和fmt/CI Clippy/strict/CLI build/严格clean。

本地接受149个不同测试，recorder另列一次维护运行。v1、calibration、v2共18条命令/36原始流；不将失败运行改为成功，也不重复累加v1中的3项P5成功。4887项源码闭包在各运行内稳定；运行间只有p5_bulk_corpus.rs的可逆六常量/说明差异。源码基础de386896b加final-source三文件构成最终编译输入。

另保存旧实现私有两项实际失败、两相关源码及其原全树清单的摘要来源；不声称归档了私有旧全树清单或其全部源码，也不冒称root旧实现复跑。本归档不含冻结CLI二进制，字节数和SHA在回执。完整workspace及本提交自身CI仍待完成。
