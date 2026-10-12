# Fresh workspace v4: P5 counted-work failure

2026-10-12 root保存失败原始观察，未接受完整工作区。136个成员（execution、metadata、134份stdout/stderr）从最终tar重读逐字核验，capture长度和SHA见manifest.json。execution SHA `1b43ee5e24daa64bc5d9534f9665d43355b3c0ead23ff1a69a69b3671ca91ff2`。

session70485已exit1。前63个测试批次成功；command65的四target构建完成，但Cargo在第一个p5_bulk_corpus的3 passed / 2 failed / 1 ignored后fail-fast，后三target未执行。runner因此另报summary/artifact cardinality mismatch；不要把未执行target计为通过，也不要误归为独立产品故障。

真实失败是两条旧计费断言：many-method-class的AnalysisSteps为12104，旧值12060；direct/shared全语料都是19955，旧值19911。三处差值均44。此前已执行的语料和所有其它已观察计费维度、输出字节相同；未定位逐方法成本前不更新基准。

4854项来源前后相同，guard_stop全部为None；finally exact cargo clean为0、target absent。该次不覆盖第二seed，也不能与旧失败运行拼接。后续必须完成归因和准确回归，再从空target重跑fresh双seed。当前状态见handoff.md。
