# 条件局部自增：已确认的控制流恢复缺口

2026-10-10 root 实际执行并独立核验该完整类基线。原源码两组、JADX default/none 四组完整源码编译和验证运行均成功；Jarde default/all 四组生成源码都因缺少返回语句编译失败，没有候选运行结果。独立核验通过不表示产品恢复成功。

原形来自 `TestVariablesDefinitions2.TestCls.test(List<String>)`：null guard 包围增强 for，循环体在空字符串条件下执行局部 `i++`，末尾返回计数。实际原始运行输出为 null=0、empty=0、mixed=2、repeat=1、null-element=NullPointerException；六条成功运行的 exit/stdout/stderr 逐字相同。

Jarde 的 `countEmpty(Ljava/util/List;)I` 是 explanation_only，准确拒绝原因是 `jre_region_arms_do_not_meet`，说明 block 0 的分支两臂未证明共同 join。泛型 Signature 拒绝是方法正文未被证明的后续结果。目前不能归因于局部 iinc 发射，也不能仅凭该说明确定触发了 region.rs 中哪一处 guard。下一步应定位 null guard、循环 exit 与 join/frame 的交互，先核现有机制，再决定是否需要扩展。

证据保存在 baseline-root-v1：31 条真实命令、112 个闭合文件。root 实际独立 verifier v3 接受物理方法顺序/flags、双 JDK 每方法全部 BCI、primary/derived 来源、default/all 整类及方法正文/映射恒等，以及四条准确失败路径。raw/argv/hash 见 results/collector-execution-root-v1 和 results/independent-execution-root-v1；接受结果为 results/independent-acceptance-luna-v3.json。准备脚本和未执行的 verifier 旧版保留。

该缺口单列，不混入当前 recover-int-field-multiply-updates。EM-23 仍部分完成，71 单元/612 测试文件分母不变。

06:53 UTC root 以同份已冻结整数CLI和同一javac23 class，仅打开既有 JRE_JOIN_PROBE/JRE_PREFIX_PROBE，真实stderr显示 outer branch block0的ipdom已为45，successors6/45，forward_else=true；没有内部branch诊断行。说明不能把当前问题笼统解释为外层join不存在；仍需定位单臂walk/loop frame与返回next的证明路径。诊断raw/argv/hash独立保存在results/join-diagnostic-root-v1，原baseline闭合清单未修改。
