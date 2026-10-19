# 固定原件 TestBreakInComplexIf 完整基线

完整上游1533B源码保留外层JUnit、TestCls、非静态Point和check()，未裁剪或改写。冻结CLI SHA `8c91115a91a31b041ecaf53a2b964bb57606342d04b524035293ae0b7accd285`。实际54命令/108原始流、16目标与Point API、20次完整源码编译；原件4VM共36观察，JADX两次JDK23整树成功重编/4VM共36观察，九个有限case逐格返回、异常类别、stdout及完整输入/after Map/Point状态相同。所有VM均parentless、实际双CodeSource核验、空cwd；三个错根控制在CASE前拒绝。异常消息/栈按原样逐格保留，不要求不同VM/debug的诊断文字相同。

真实缺口：八组目标test(Map,int)均fallback，构造器/check与八组Point构造器均structured。16次Jarde完整重编均失败：八次JDK23缺return，八次JDK8另有原始依赖major55/52边界；没有Jarde候选VM运行。Point原件真实ctor含外层参数、x/y/this$0完整保留，不能把这项循环缺口归因为缺失Point。目标test来源仅7/33，其他构造器/check/Point分别3/3、75/75、12/12；default/all八对正文、质量、identity和maps均同。

独立审计复开13131条pin；root额外核全部八个实际VM的wrapper/JDK/classpath/roots/空cwd及错根先于CASE。初次前置路径错误、旧方法库存中止、未执行误覆盖草稿、审计诊断/路径比较错误及实际打包前置失败均保留。原件/JADX结果是新跑，不能把这些接受事实称为修复完成。下一步是同轮真实IR/ForHeader/首失败路径，不凭此基线新增机制或关闭CF09/71/612。

归档复用已接受的通用Continue文件格式及原packer，不新增打包机制；包括精确CLI的既有构建收据/原始流，CLI二进制被唯一明确排除。只保存regular payload和inventory，空运行目录已在现场和driver中核验。root逐成员复开bytes/SHA/mode，计数与归档SHA见root-review.json；capture内baseline/root-review.json绑定实际基线接受收据。
