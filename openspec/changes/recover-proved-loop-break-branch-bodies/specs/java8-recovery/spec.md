## ADDED Requirements

### Requirement: Proved conditional break bodies retain their complete loop effects

系统 SHALL 恢复头测循环中有完整证据、单入口无环且所有路径通往本循环同一出口的条件 break 分支体，保留各条件、赋值、循环更新和物理来源。系统 MUST NOT 因分支体不属于自然循环集合而将其写为空臂，也不能只写一条 break 丢弃分支内效果。

#### Scenario: Original Loop4 retains conditional multiplier assignment

- **WHEN** 对冻结的完整 TestBreakInLoop4 debug/no-debug 原件执行 class/JAR、default/all 反编译
- **THEN** 构造器、test、check SHALL 全部报告 structured；test SHALL 保留 for 初始化与更新、对 a/b 的条件赋值和本循环 break，且不留 17/23/29 的 uncovered 引文。完整生成源码在具备适配框架依赖的编译器下原样重编后，隔离 VM 对点号、a、b、c 分别返回 1、1024、1024、1；原 check 的实际结果及框架版本边界须单独如实记录

#### Scenario: Shared entries and nonclosing exit graphs remain refused

- **WHEN** 候选分支体出现外部额外入口、不同出口、循环回边、未证明的异常边、已占用块，或不能完整表达其控制流
- **THEN** 系统 MUST NOT 将该体发布为已完整恢复的循环；未被其他证明拥有的原始块、拒绝原因和物理事实 SHALL 保留，不得通过减少覆盖分母或静默去重隐藏缺口

#### Scenario: Break origins and bounded stopping stay faithful

- **WHEN** 恢复该分支体成功，或在证明、复制、遍历和发布途中遇到预算不足或取消
- **THEN** 成功输出的 break SHALL 指向本循环并保留实际条件/跳转指令来源；停止 SHALL 传播实际原因和用量，不发布部分 owner、不将停止伪装为普通形状拒绝。default/all 的正文、identity、quality 和源码映射 MUST 保持一致，映射引用须为有效 UTF-8 边界和物理方法范围
