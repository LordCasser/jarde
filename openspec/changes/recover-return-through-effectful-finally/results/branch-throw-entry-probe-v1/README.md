# EX.finThrow 真实入口探针检查点

测试提交cecfb59a4仅增加cfg(test)探针，未改变生产恢复规则。实际4命令均exit0，1 test passed/0failed/0ignored；4946来源前后相同，默认stack、资源守卫与exact cargo clean完成，target不存在。完整argv/cwd/exits/8 raw、编译的guard源、原EX输入、静态接缝分析和36项/0错误独审在便携包中。格式化preflight单列，不计作新功能测试。

实际四块0/4/14/27、四边，0只有两正常边、没有exception edge。直接examine@0为Claimed structured Finally；@4/@14为Refused；直接copy proof为Some，保护0..17、正常cleanup17..25、handler28..36、saved16→26、primary27→36→37。结合现有leaving_edge/dispatch源码可定位入口漏调度。test对部分物理事实作assert，其余证书/verdict是打印观察，不冒称回归断言；helper使用empty init Sites，未运行真实Region/Builder。

下一步、限制和命令见仓库handoff.md。unexecuted-draft中是尚未编译/运行的独立完整fixture和6输入driver草案，没有实测oracle，不得把它们当恢复成功或已冻结永久输入。原完整EX和父任务4/7保持，未归档。
