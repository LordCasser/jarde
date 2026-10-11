# 默认栈诊断（未验收修复）

focused-v4 实跑 Reader 单测通过，默认线程 API 首个测试 stack overflow/SIGABRT；cargo exit 101 后 clean 0，target不存在。完整编译 source closure、真实命令/env和失败binary pins见focused-root-v4/execution.json。binary保留外部 tmp，不将85MB本体写仓库。

LLDB v1 参数错误无执行，v2执行并崩溃但无bt，v3使用on-crash命令取得真实45帧：Builder::region嵌套→block→instruction→render_value_definition，Region walker不在当前crash链。LLDB exit 0 是debugger退出，不能解释成target通过。机器码prologue各命令见两份frames/execution.json；Builder::region 202512 B（约198KiB），render_value_definition 42512 B，prepare_physical_class_source 272336 B，仅当前ARM64 debug构建单帧。

此前8MiB诊断已归档于 recover-proved-nested-loop-side-exits/results/stack-diagnostic-root-v1，仅用于排除不终止递归；其API map比较失败属于测试请求essential无SourceMap，已修正请求。大栈不作为验收。
