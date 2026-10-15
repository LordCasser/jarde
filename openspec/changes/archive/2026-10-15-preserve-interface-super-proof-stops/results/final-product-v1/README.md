# 最终产品证据

产品 `c8718aeff6cc08b686c7b3c3495bfd0bb9efa2b1` 的 [自身 CI](https://github.com/LordCasser/jarde/actions/runs/38692921405) 已成功，4 jobs/52 steps；两固定 seed 各367 suites、3615通过/0失败/98忽略。105个指定回归逐seed唯一通过，显式JDK/full-class六命令九项测试也实际通过。strict为该提交快照365/0；后续规划或同步归档导致的规格计数单独验证。

`own-ci.tar.gz` 是该精确提交的共同CI补充包，两片保存相同字节；含163 API observations/326 raw、完整logs.zip、root848项及独立1644项验收、105门来源和原始失败历史。历史root-local-v1未覆盖或重打包，原receipt中的pending字样保留原时点；本目录新验收说明其最终状态。包内manifest逐文件核hash，外部package-independent-review再核原始bytes，3504检查/0错误。

`interface-final-local.tar.gz` 保存最终2669 CLI的178命令/356raw、144份编译与报告产物、8合法完整源格（12行原Runner oracle）与16拒绝格、两个真实ClassHeaders=2 CLI停止（exit4），以及4902源/Git/构建绑定。此接口合同使用受控JDK23，不冒称双JDK。原fixture、Runner、oracle及生成Java未修改。

原第一次独立CI审计因setup/Post标题匹配而拒绝、打包v3/v4因JSON层级和文件名假设而预检失败，均保留；修正只定位实际记录，不改产品、不重跑或放宽产品断言。原CLI/JDK二进制与target不入包，按pins识别。更广覆盖和direct-parent/reference邻居不由这些结果证明。
