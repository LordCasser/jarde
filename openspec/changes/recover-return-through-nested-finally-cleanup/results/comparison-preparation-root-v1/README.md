# 完整对照准备，尚无运行验收

Luna准备，root逐段审查并修正JADX失败不得中止、原始Code/row/35BCI核对、完整EX driver及完整source失败状态。durable runner在tools/compare.py，root改为仓库相对定位，不依赖私有/tmp目录。输入源码草稿无损归档，尚未放入tests/fixtures、尚未编译。

runner将双编译器独立完整类default/all原样重编并在双JVM比较18case；完整EX保留六方法，单独driver仅调用nestedFin。若EX可编译，运行同一18case；仍保留finReturn fallback，不能因此称其结构恢复完成。JADX保持真实生成、编译或运行失败。

工具存在、Python语法检查及静态classfile检查不是目标编译或运行证据。实际用到的CLI、JDK、JADX、原始class和生成source均须在每次execution.json固定；root仍须独立核所有method/BCI/身份与rebuilt-only classpath。

root后续用实际原始stdout修正日志空格解析，最终工具还改用封存EX.class原字节作为EX运行oracle。旧准备包与实际baseline-v2/补充sealed oracle的范围分别保留，运行时以tools/compare.py当前源码及execution.json实际pin为准。
