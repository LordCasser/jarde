# 独立待复现架构债务

Luna静态审查发现普通continue_prefixed_loop_arm在region_at先claim child后，后续helper拒绝时可能留下visited，而caller gap只含branch/prefix。该生命周期代码早于本片；静态路径不能直接证明公开源/覆盖错误。原NL5旧baseline已实际保留全部29物理BCI。Stop通过Result直接传至请求层，不发布source/map；不以visited未回滚就宣称公开Stop错误。

待独立构造公开反例，核最终fallback builder/整方法报价以及来源，再决定需要回滚还是保留已走region。当前helper本身只读所有权，局部证书消费不引入事务机制。若当前新增门实际导致完整Java或完整覆盖假阳性，则按实证纳入修复；否则不混入当前语法点。见私有review `/private/tmp/jarde-proved-loop-arm-returns-review-luna-v1-20261018/review.md` 的静态线索。未增加CF11或71/612完成数。
