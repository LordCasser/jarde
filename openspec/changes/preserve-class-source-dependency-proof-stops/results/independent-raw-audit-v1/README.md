# Snapshot proof CLI：独立只读审计

确认两处真实预算停止没有进入最终执行平面：cap=8动态trace定位到snapshot hierarchy proof；cap=15另行定位到方法正文成功后的double-brace projection。两次最终class仍Complete、CLI退出0。不是仅凭解释性正文倒推某个helper，也不是七路径测试矩阵通过。

冻结CLI为c814所记录的59602720 bytes，SHA cdf06694…；实际文件及emitter-focused-v2 CLI pin一致、mode0555。facade/budget/CLI task三相关源码SHA同时匹配c814 git blob与focused源manifest。JAR为4469 bytes、SHA1ee63f38…，13 entries的CRC可读，H1及相关内部层级class均存在。没有把三源码关联冒称全native/source逐命令身份验证。

六次正常CLI参数除了class_headers完全相同；JSON生效limits也证明只有该维变化，排除了覆盖未应用或改request/evidence/input。view、class/member原始身份与结构coverage相同；前五方法去elapsed后全文报告相同；wide、cap8、cap14的main analysis也相同。

| 正常运行 | exit | class execution / headers | main结果 |
|---|---:|---|---|
| wide（limit4096） | 0 | Complete /16 | Complete，contains_statements，含完整引用cast |
| cap1 | 4 | Partial ClassHeaders /1 | Refused，明确budget_exceeded_class_headers |
| cap2 | 4 | Partial ClassHeaders /2 | 同上 |
| cap8 | 0 | Complete /8 | recovery Complete，explanation_only，7处缺引用转换证据，无budget诊断 |
| cap14 | 0 | Complete /14 | recovery Complete，explanation_only，仅剩BCI82的H1$ViaSub→H1$Greet证据缺失，无budget诊断 |
| cap15 | 0 | Complete /15 | main去elapsed后的全部报告与wide相同，正文及class正文逐字相同 |

正常六次各少于0.03秒。raw/argv/时长/退出码与摘要、逐member usage/status/诊断在audit.json和trace-review.json；usage是原报告累积快照，不是逐member增量。main文本及相对wide差异单独保存。输入并未丢失；wide与受限请求使用同一个冻结JAR，低cap的显式停止也是预算接线的正控制。

新增调试材料提高了归因强度，独立重读了脚本和全部raw：

- 无效debug v1：`run`只有重定向，重置了参数；inferior为cli_request_json/EOF、usage0、退出01，零quota断点。GDB进程exit0不算产品成功，此次完全排除；原五次正常probe不受影响。
- 有效cap8 v2：bt16有5次quota命中，前四只见到selected_reference_header，单独不足以定位后续helper。
- 有效cap8 v3：5个真实Budget::exceeded，各有30 frames。前四完整经过charge_header_attempt→resolver→selected_reference_header→snapshot_header_chain_widens→prove_snapshot_hierarchy_widenings→snapshot_hierarchy_widenings_presented→member recovery；第五经过prove_double_brace_allocation→project_class_source_double_brace。
- 有效cap15：恰1次Budget::exceeded，完整double-brace projection链，无snapshot frame；main已经正常完成但class仍Complete/exit0，分离出后置类projection的停止丢失。

三个有效debug inferior都正常退出；各自**全JSON**仅剔elapsed后等于对应非debug结果，limits含elapsed限额先逐字核对，不掩盖其它差异。脚本只配置断点、printf、bt、continue，没有改返回值、变量、调用函数或跳转；GDB工具hash/保存version材料也核验。bt证明预算拒绝发生于指定调用链，但没有打印每次BCI/owner参数或随后的Result值；四次命中不等于四个唯一站点。不能据此断言四个wrapper的Err catch执行了。

源码说明另一条关键损失路径：resolver的非Complete execution合入proof局部execution后返回Ok(None)，而proof只返回Vec/Option；局部停止未交给最终报告。wrapper把BudgetExceeded/Cancelled变空证书的静态分支同样有风险，但reference-overload、direct-parent-accessor、direct-parent-field并未在这批trace中被动态证明。cap14也没有自己的trace。

现行analysis-contracts要求如实报告预算造成的停止、逐方法/类顶层汇总执行，并由CLI按非完整报告退出4。java8-recovery允许证据不足时合法交付explanation_only；**解释性content本身不是违规**。这里已有真实quota拒绝动态证据，不能把该停止换成普通“无转换证据”后仍称Complete。结构coverage可继续完整，不能为了体现停止而伪改已读范围；cap15也表明正文成功不代表本次所有请求proof完成。

audit.json保留原五次raw单独能证明的范围，trace-review.json明确升级后的组合证据；原限制没有倒写成历史动态事实。classification.json的SHA24b3f6ea…只作父级摘要交叉绑定，结论来自实际raw/栈核验。此次审计没有运行CLI/GDB/Cargo/JDK、编辑原capture/源码或查看当前target；没有修复、workspace、CI、编译执行等价或七路径通过声明。
