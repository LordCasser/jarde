# Candidate-v1 事后离线裁定

本文件只对已保存的 candidate-v1 输出做离线重裁定。12 行代表 SameErasureBinder、MultiUseResult、IncompleteSite 各四腿；它们是旧 CLI 的事后裁定，不是 candidate-v2 重放，也不改写 candidate-v1 原统计。未调用 CLI。

Candidate CLI：`/private/tmp/jarde-generic-calls-candidate-v1-cli`  
CLI SHA-256：`b613c0482c8aa56bdf479321abbcc6cadd12c500dc9eec6e2efd8a62daa00900`  
固定 matrix manifest：`openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v1/gc01-08-140/manifest.json`（SHA-256 `a2f3ef31dad2c5e4c627d5037bfeebf40bda4a377febe93411f33f6e8fe1160a`）  
嵌套输入 manifest：`openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v1/nested-call-4/manifest.json`（SHA-256 `59ce83e55b7b013182c7042158ca341b6ad73360e7a2eb7abda2211dba6ea311`）  
判据脚本快照：`candidate-acceptance-replay-criteria.py`（SHA-256 `a19895e70ef306a3024132a3da7c07cf891ea87e5ef0e25e01413989514ce836`）  
冻结输入 manifest SHA-256：`c01c7f3a42e1c5e759ac45e18d13d16e233df54dfc29ae2e32c27200a826496d`。

离线完整性检查重新核对两份既有输出 manifest 中列出的文件；没有重新编译、运行 Probe 或调用候选 CLI。

| 族 | 旧 CLI candidate-v1 四腿 | 离线判据通过 | 完整反射相等 | 编译 | Probe | 行为 | 裁定类型 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| SameErasureBinder | 4 | 4/4 | 0/4 | 4/4 | 4/4 | 4/4 | refusal control |
| MultiUseResult | 4 | 4/4 | 0/4 | 4/4 | 4/4 | 4/4 | refusal control |
| IncompleteSite | 4 | 4/4 | 4/4 | 4/4 | 4/4 | 4/4 | feature recovery |

## 逐腿裁定

| 族 | JDK | 模式 | 结果 | 类型 | 完整反射 | Jarde 源 SHA-256 | 原输入 JAR SHA-256 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| IncompleteSite | corretto8 | debug | pass | feature_recovery | 相同 | `56fc806f2c0e6b908118f83d0b6ed0e4bd286205a7f9fcd311d5b25f1d2ef02f` | `8caf43e2252d114b114d51193e5cd3153fac096f13a48889fc4e88c4f8a1a27b` |
| MultiUseResult | corretto8 | debug | pass | refusal_control | 有差异 | `c4e28f41a7800349c7a11f24c4f38629adae9f3245497436a2c7ab78cddcadc9` | `26c42ab150ae61d478583cc9bb1bd4076dde60efa50202ff56491f95d78531f8` |
| SameErasureBinder | corretto8 | debug | pass | refusal_control | 有差异 | `a74e0070fe9588f94ddb25c1c476fde85e8b592dee41d3472eab0eb55082462e` | `d2a8ba5de0f4b7d2da306df5f7b11932014b1616c734ef6488acc9ac65cf31ad` |
| IncompleteSite | corretto8 | nodebug | pass | feature_recovery | 相同 | `c22257bfc7fa7dddd45bd7a310887b51506168edcd590403527efe7e205952b1` | `c659525e0fa7cb64236e08ea4c643bf182881786617c4fea5ab820fb9405d55e` |
| MultiUseResult | corretto8 | nodebug | pass | refusal_control | 有差异 | `2ab22e6df6c7a4071b20120f31b10ceaaac1ef9bdd7f6018585cd069c333060a` | `8d5b40d975a71acc2abe5d9d6a33f6b845adfc20dfa209bfc07fc045806e1fd7` |
| SameErasureBinder | corretto8 | nodebug | pass | refusal_control | 有差异 | `e50214d2a92ed46bdb29584ce512c08687b59232722b6c9526ffe29adaf2cb99` | `919b9c4fed9fd5e73b5edb7f2ecea834a67c2964568d910c6f040267f6260ab8` |
| IncompleteSite | openjdk23 | debug | pass | feature_recovery | 相同 | `56fc806f2c0e6b908118f83d0b6ed0e4bd286205a7f9fcd311d5b25f1d2ef02f` | `26c16dc2130a6a63193a0535016e519b2f048733d9b68cadef7950ba50bdb94f` |
| MultiUseResult | openjdk23 | debug | pass | refusal_control | 有差异 | `c4e28f41a7800349c7a11f24c4f38629adae9f3245497436a2c7ab78cddcadc9` | `08749c04b3dcce9459e131384ab38e40aeb1da1514aaf4c9f8a4eb0c99ef0cb3` |
| SameErasureBinder | openjdk23 | debug | pass | refusal_control | 有差异 | `a74e0070fe9588f94ddb25c1c476fde85e8b592dee41d3472eab0eb55082462e` | `6a4836c9be87000f4f6959415ce47b452ced7e25c64c79ca64de66ff3c572478` |
| IncompleteSite | openjdk23 | nodebug | pass | feature_recovery | 相同 | `c22257bfc7fa7dddd45bd7a310887b51506168edcd590403527efe7e205952b1` | `52dc3a3143e5b2e20632bcfb592cfcd8a59055c664bc5b6276c5c763f864c7fd` |
| MultiUseResult | openjdk23 | nodebug | pass | refusal_control | 有差异 | `2ab22e6df6c7a4071b20120f31b10ceaaac1ef9bdd7f6018585cd069c333060a` | `177ae21dbe67d969dae604f092c0e6c6acbab5a6585cc04f62cf6f4fd2899361` |
| SameErasureBinder | openjdk23 | nodebug | pass | refusal_control | 有差异 | `e50214d2a92ed46bdb29584ce512c08687b59232722b6c9526ffe29adaf2cb99` | `571cfc3f8df4f5d139159984f7a9db618e6ec86ee01f3df9ff3a8a135d16edd1` |

SameErasureBinder 的完整反射保留为独立指标。拒绝控制仅在候选保留类 `T`、实际物理 `sink(Number):Number` descriptor、反射 raw `Number` API、显式拒绝 marker 以及原有 null-call 行为均成立时通过；完整恢复 method `U` 分支同样需精确恢复 bounds/binder/API 并满足源与行为证据。拒绝控制不会计入 API 恢复数。

IncompleteSite 属于合法的单调用条件正例；本报告只使用已冻结的四腿单调用 census 与实际反射/行为结果。它不证明普通源码能产生 partial census，部分 inventory 的拒绝仍需独立 unit/transaction 证据。

完整性错误：generic=0，nested=0。

每行完整 JSON 证据、差异明细、源与 Probe hash 见 `offline-disposition.json`。原 candidate-v1 目录未修改。
