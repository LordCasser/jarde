# Seq2 raw方法来源范围补核

root巡查发现原Seq2 baseline checker用whole-class text的长度/UTF8边界校验方法source_map。map实际属于各自RecoveryReport.text，原物理BCI/owner集合检查有效，但文本范围证据过宽；旧收据/工具不覆盖改写。

本包补核脚本只改为逐raw方法text边界，并将raw text加入default/all对照。root实际执行接受同一冻结receipt的8请求/24方法；原始构造器/ascii/clinit文本分别231/843/307B（debug child），不再按whole2230B计算。两项内存变形负控——越过raw方法但仍在whole内的span、UTF8 scalar中间起点——均真实拒绝；它们不是JVM变体。原始/生成源码、CLI和原始8次/生成72次历史运行未改、未重跑，不新增运行case计数。

归档5成员，SHA256 `09528fc51d0514ec0686dcfde72a23d05b9316f26252919bc22bc95d04df2ab6`；已经新typed foreach结果中的baseline-archives-root-review-v1.json独立逐字复开。本次范围补强不改变Seq2实际语义结论，也不虚称投影后的whole/method文本具有同一map。
