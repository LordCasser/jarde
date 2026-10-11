# 共享值 SSA 与永久 API 验证

root实际passing v3五命令：fmt、真实SharedStoreInput SSA单测1/0、p3_for_latch API8/0、terminal_return5/0、clean。全部4700完整源码/包含输入前后恒同、一秒5GiB free/target1GiB守卫未触发；结束target不存在。真实SSA确认isub@4经dup@5产生两个不同def，各仅一次准确消费；准确返回叶证书允许ForHeader init@9候选，但公共Builder不发布初始化子句，仍可靠拒绝，测试没有把源级共享冒称同一SSA双use。

unit compile failure v1是测试用错CanonicalBlock.bci接口（正确id().bci()）；API compile failure v2是测试用错私有Segment字段（正确start()/end()）；仅修测试调用，不改生产恢复或Builder。v2的SSA1/0也是实际通过，但不重复计覆盖。全部失败raw保留。

root独立checker v1只因误写4699而失败；准确包含controls/shared两个新增编译输入，真实数量4700。完整workspace已clean后，root实际执行checker v2 exit0接受5命令/10raw/4700输入与1/8/5测试结果。formatter实际运行后receipt序列化用错相对path导致程序失败，raw保留；后继passing v3 fmt --check真实通过。

复制保持receipt和raw原字节/原路径，inventory绑定全部复制。此目录不作为完整workspace、自身产品CI或最终clean main交付。
