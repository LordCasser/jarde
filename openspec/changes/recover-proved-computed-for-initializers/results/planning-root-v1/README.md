# 规划与当前冻结基线

本目录保存实际 bafb CLI 的两个现有恢复复核、私有诊断方案及 OpenSpec 规划执行。CF07 counted@20 与 CF13 walk 旧缺口已通过当前完整类双 JDK 对照：16 编译/运行命令、4 paired runtime legs；精确 argv/source/input/raw/工具及 If span 独立验证通过。没有 fresh JADX 执行，也不表示整个 CF07/CF13 单元关闭。新 computed-init for 仍输出 while；10 项实现任务全未勾选。

前稿 triage 的 counted/walk 待实现判断被拒，纠正稿和实际 baseline 保留。runtime v1 由于精确 span 有前导空格而在编译前失败；v2 只修此断言，并实际完成16命令，root独立验证器加核完整 argv集合、复制源码/runner、original class SHA及全部生成class。原失败不覆盖。独立程序的原路径指向 /private/tmp；归档为字节不改写拷贝，执行与文件 SHA在原记录中可追溯。

architecture v3 的 arg-1 专用模板被 root 否决；v4改用原Builder初始化语句，但材料化、作用域和异常上下文仍须真实IR诊断。这不是已完成生产实现，task1.1/1.2候选由Luna私有准备，root工具链串行验收。
