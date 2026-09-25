最初独立样例在被恢复 Probe 内直接 athrow。当前裸 throw/new Throwable 被既有恢复边界拒绝，使方法缺少返回语句而编译失败。此处保留输入与完整实际输出。主样例修改原始源码，把副作用与异常 helper 移至独立 AuditEffects；重新编译原始输入、再运行 CLI，生成的 Probe 整类不做删改。这个边界不由参数类型修复处理。
