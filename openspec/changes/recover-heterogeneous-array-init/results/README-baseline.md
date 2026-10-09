# EM-18 当前基线与架构证据

基线 main 为 `4fba93438acd444c4ce0f5d916a05f9af72caa28`；CLI2 SHA-256 为 `f2ad93f5f024a0e1ec7d74e916839b17cac0f8db6fdf8c1a1eb62051f6dde33b`。全部原始文件保存在 ../evidence/baseline-20261009.tar.gz，归档 SHA、大小、源码 hash 和原临时路径映射见 baseline-archive.json。解包得到 jarde-em18-baseline-20261009；raw manifests/argv 的原始绝对路径未改写。

18 条有效 family/JDK 腿：原程序行为有效，Homebrew JADX 1.5.6 与源码参考 checkout 2fb1b16386941660fda07e9017285aec40fcb37f 的本地 JADX 均完整重编/验证运行、stdout+stderr 匹配 18/18；Jarde 行为匹配 0/18。四条 Jarde exit-0 腿丢失 stdout 且保留 initializer 拒绝，不能计作恢复成功。root逐文件/命令双流校验 primary512files/156commands、addendum02 111files/36commands、本地JADX219files/61commands，校验结果无 hash 或隔离路径问题。

初始 BigDecimal 源码错误、JADX 初始无包入口错误、addendum01 不完整 ledger 和 local runner01 schema 错误均保留。合法 BigDecimal 与精确冻结 CT.class 采用 additive02；JADX 运行使用实际生成 package，再用源码参考版完全重放。以此处的双流统计为准，早期 patrol-summary 的“编译运行”不是语义成功率。

当前最小类型改动复用现有平台兼容事实、数组形状和 Runtime-selected bounded snapshot header walk，按实际 aastore BCI/source/component消费。snapshot target 可由可信 header 直接点名，中间 header 不可猜测；aastore自身不陈述数组具体分量。

构造元素还有独立的 new@1/ArrayInitializers 证明组合缺口，详见 em18-constructor-composition-audit.md。main 的 arraylength/concat/field 丢失也是独立失败。类型证明片的完整 factory-family 正例与 direct-new 未覆盖控制分开构造，原18基线仍完整重放，不能靠删除成员使它通过。EM-18 整单元保持待扩验/未闭合。
