最小编译修正草稿

首轮错误对应 switch_region 中两个外层变量引用。已有绑定名是 fall_throughs；仅替换两个调用点的外层标识符为 fall_throughs，保留紧随其后的 closure 参数 fallthroughs 及 closure 体，不改其他内容。

按上下文定位：
1. proven_fallthrough || local_loop_join 分支中的 frame.switch_arm(...)
2. else if join_node.is_some() 分支中的 frame.switch_arm(...)

见 caller-name-fix.patch。
