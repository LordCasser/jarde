# 完整边界类的真实 public IR

root 在 typed 冻结生产源码不变的条件下用 public reader/JVM API 实际读取两完整原 class 的五方法，1/0/0。10 个 profile 的六阶段 complete，物理解码连续且每个 opcode 有一个 canonical 块归属，SSA 块身份闭合。独立 root verifier v2 退出0；这里只接受观察，未测试新条件证书或叶。

两 JDK 的 layout/全 canonical 边/物理 handler语义相同：partialBreak 7块9Normal；innerLoopBreak 10块13Normal，有57→38回边；innerSwitchBreak 8块10Normal；terminalCase 6块6Normal；caughtExceptionThenFallthrough 9块14边，其中40/45/47→60各一Exception handler0。首次 verifier 把不同 class 的 catch_type_index 24/60直接比较而失败，root v2读取各真实常量池Class/Utf8后比较同一NumberFormatException语义，保留原数值/raw，未弱化其范围和ordinal。首次执行包装脚本因嵌套换行语法错误未启动工具链，之后修正另存。

临时integration test运行后已删除，76唯一路径/77分类 frozen pins 保持；target峰值180559202bytes，cargo clean 实际348files/172.2MiB，target不存在。私有函数提取 proposal 仅方案，未应用或编译。JSON历史绝对路径保持，全部拷贝SHA见copy-manifest。当前条件片仍1/7。
