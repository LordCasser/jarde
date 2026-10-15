# Class-view ambiguity：独立只读复核

接受这份有限公共 API 失败复现，未发现材料或规划的实质矛盾。实际是私有 f014 加既有13工程候选及b973覆盖的组合，不是primary运行或生产修复。

四条真实命令退出码为0、0、101、0：仅新文件rustfmt；controls筛选命令；duplicate exact负例；严格两参数cargo clean。8个raw流、argv/cwd、返回码、guard和源码snapshot的字节/SHA均独立核对。没有运行原作者会写原目录的自核脚本。

- controls实际2 pass/0 fail/1 filtered：唯一成员测试完整执行abstract、native、Code三子场景并保持NotDeclared/NotDeclared/Read；foreign-owner测试保持原错误码与零body/code。
- negative明确Finished test profile、Running test及实际0 pass/1 fail/2 filtered，退出101；panic处为格式化源码204行。它返回真实Performed/ClassViewReport，body=NotDeclared(Abstract)，列举中仍有index0/1、flags1025/9、相同owner/name/descriptor。
- 同一首行的Name请求在此前已通过Ambiguous双候选、零method_bodies/code_bytes断言；这是由已到达panic行及前置源码断言证明，未捏造单独JSON输出。
- 九行矩阵只抵达abstract-code首行、首个listed identity。其余八行与第二个identity没有执行，不能算覆盖；负例Method的后置候选/预算断言同样尚未执行。实际错误报告本身记录body/code/IR均为0。

历史源码3563→3564唯一增加class_view_ambiguous_identity.rs，格式化仅改该新文件；测试及clean前后集合相同。3563原pins也逐项等于上一interface复现最终快照。13工程pins、build b973及已有interface Stop测试f285保持；格式化测试与test-only patch内容完全对应。测试内构造物理class字节，无外部fixture，不声称这些重复声明通过JVM verifier。

历史clean严格argv=[cargo,clean]、私有cwd、exit0、无guard stop，raw删除672 files/345.7MiB。runner当时记录target_absent=true及owner released。清理的资源阈值明确放宽为minfree0/max2^63-1，以保证收尾；这只适用于cleanup，三个工作命令仍按1秒5GiB free/1GiB target。这里既没有把cleanup描述成按工作阈值执行，也没有查看当前target。默认栈、stable/offline/jobs1/debug0/testthreads1和相关unset环境已核；未新增工具链执行。

只读规划复核见planning-review.json：主规范目标选择requirement的description及六个原scenario逐字保留；新增四场景约束完整成员表的重复identity、Name/identity一致歧义、foreign-owner优先及唯一body种类。复用既有候选，保留index/flags/顺序且不按相等identity去重，在prepare/decode/body/code/IR前终止歧义，范围合理。规划明确保留partial-table/shared-budget、reader/JVM API、逐record class-source及CLI身份schema；CLI对照属于回归验证。四项tasks全未勾。artifact strict由父任务处理，本审计未执行或替其宣称通过。

audit.py、audit.json及manifest固定本次只读结果。没有修改原材料、planning、checkout或index，没有工具链和target动作，没有新增mainroot门槛；不以本次失败复现替代修复后的完整矩阵、workspace或CI。
