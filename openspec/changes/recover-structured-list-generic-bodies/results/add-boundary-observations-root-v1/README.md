# add 实参边界的原始观察

本目录保留 v6 CLI 的 inner-option 与 null-add 两家族 debug/none 双 JDK 原/JADX/Jarde完整类记录、输入、候选补丁及真实预算探针。它是修正前观察，不能冒充修正后泛型产品通过。

inner-option 两份各29命令，原/JADX/Jarde完整类编译运行一致；identity runner 特意使用两个独立 new Integer(5)，原类和生成类都得到 CME，追加引用等于内层元素而不等于外层元素。Jarde 仍 raw + Object包装。root 查明 assignment_loop_matches 必须等于1会误拒内层元素的两个合法 List 祖先；同局部唯一物理写、next/checkcast/store/load/add SSA 和精确元素赋值证明仍保留，只需要至少一个匹配祖先。

null-add 两份各35命令，两个完整目标类分别为 List<String>/List<Integer>。每份8组JADX/Jarde完整生成类编译运行与原类逐字一致，Jarde 两方法仍 raw + add((Object)null)。四条 oracle 行记录匹配追加null/CME及不匹配清空分支、集合状态和effects。新增候选复用已有 AST Null、物理aconst_null和唯一SSA实参用途、精确add(Object)平台契约和现有投影事务；尚待新API/CLI验收。

late-stop v6 六条真实CLI命令 exit4，预算32531时 analysis_steps=25195、请求8008，发生于最终List用途重新证明的整块计费；第一次相同证明 charge 的位置是16313。泛型声明/正文未安装，原始RecoveryReport正文和map保留。33554停止已在overlay完整安装之后，不能错误要求所有Partial都退为raw。永久API测试正在串行验证。

独立债务：预算33427时既有 enum_projection_stop_error 把 StopReason.written 当成 consumed（消息12151，而实际analysis33379），requested固定1；保持停止维度/结果但诊断用量失真。当前有限List证明直接Budget错误的8008收费不经过该转换，不混入本片泛型修复。

压缩档的receipt外置路径原样保留，archive-pins记录压缩byte identity；实际命令/原始流由receipt核查，不改写成通过。
