# 静态初始化块终止语句的独立缺口

CastAudit 的 `static Object value = "ok"` 产生 `<clinit>`。class-source 已把成员声明写为 `static`，但正文仍含普通的 `return;`，得到 `static { CastAudit.value = "ok"; return; }`。完整真实输出经 javac 拒绝，错误为“返回外部方法”；source 与失败日志随目录保存。这不是引用转换引入的新指令，也不能在 cast 测试里手删 return 后声称完整成功。

当前路径为 `class_source::spell_method` 的 `<clinit>` 声明拼写，加 `build` 的普通 Return 节点和 emitter 的统一 return 语句。`DeclarationForm::StaticInitializer` 已有，无需再添一种方法类别。后续应研究初始化块的正常结束如何投影到已有块闭合，同时保留终止 BCI 的来源。

最小可验范围是最外层最后一条无值 return 代表正常结束。不能全局删除 `<clinit>` 的所有 return：分支内提前退出会跳过后续效果，直接删掉会改变控制流。应先构造直线、条件初始化和含异常/循环的反例，再限定可以投影的位置；不在 class-source 字符串上删行，不伪造缺失来源。尚未建立实施任务，单独排队。
