# 双构造器实施前基线

基础两常量枚举变更已完成 Root 独立验收，见 [verification-root](../recover-proved-enum-constants/verification-root.md)。Root 再从当前共享工作树构建 `jarde-cli`，可执行文件 SHA-256 为 `e64d6f9f9513e2bb8f07c03ff26c177c07f73cc3ceecaf0c163e70d1a14f07a0`；从冻结的 `DelegatingEnum.java` 分别以 `javac --release 8 -g`、`-g:none` 生成 class，并以该 CLI 的 `class-source --policy single-class` 读取。

当前 `-g`/`-g:none` 输出正文 SHA-256 分别为 `1aec24f5daef893e2ea09dcc48198ddc57a175088d1132838ed8334b9031b7e4`、`5f9320bca7b8fa9bf1b27b5bb25e7e34b66ac5eb8554074d34c885448237b195`。两份源码都仍把 `ZERO`/`ONE` 写作普通静态字段，保留 `$VALUES` 和 `<clinit>`，物理构造器分别含注入的 name/ordinal 参数。完整类的 `javac --release 8` 均在 `public static final DelegatingEnum ZERO;` 报“此处需要枚举常量”，退出 1；没有发布部分枚举常量投影。

此基线只说明已验收的单构造器证明仍拒绝双构造器输入；原/JADX 完整类的重编、反射和运行对照已由 [三方证据](../../evidence/java-syntax-2026-09-25/enum-constructor-delegation/analysis.md) 与 [九组拒绝控制](../../evidence/java-syntax-2026-09-25/enum-constructor-delegation/negative-controls/analysis.md) 另行冻结。
