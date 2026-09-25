## 1. 冻结完整类与位置边界

- [x] 1.1 冻结自写 Java 8 两类、class SHA/`javap -v -c -p`、完整 JADX/Jarde 源码及 JSON；root 从独立复制目录重放，三套完整源码均 Java 8 编译与 `-Xverify:all`，原/JADX 四行 `true / true / 1 / 5`，Jarde `false / false / 0 / 5`，结果与源码/class 哈希见 `member-annotation-uses/generated/summary.json`。
- [x] 1.2 `member-annotation-uses/boundaries/` 已冻结 `long`/`double` 宽槽后注解参数、末尾 varargs、运行时不可见成员注解及不同位置同类型合法 Java 8 输入；另以明确标注的 classfile patch 构造同位置重复与参数属性 2 组对 descriptor 3 参数。root 在独立复制目录重放，除 `javac` 临时路径外 summary 一致，合法/两补丁均 `-Xverify:all`，三个方法的 Code 字节哈希在补丁前后相同，属性跨度和预期拒绝见 README。

## 2. 复用类级注解树并处理成员归属

- [x] 2.1 在 `recover-class-annotation-uses` root 验收之后复用其 reader 注解体读取，增加 visible/invisible 参数属性的 `u1` 个数与每位置有序列表，沿用预算/深度/取消/精确末端；reader 正负及无属性零额外内容读取测试通过。
- [x] 2.2 将字段和方法自身注解与现有 ConstantValue、Exceptions、AnnotationDefault 在同一成员读取中交给 class source，在声明前按条输出完整注解，JSON 保留原壳/拼写/拒绝；验证字段/方法反射两行、默认/完整 evidence 文本及无注解普通成员回归。
- [x] 2.3 在 `arguments` 按 descriptor 参数位置放注解，不改变 slot 命名及正文；验证 1.1 参数反射，及宽槽和最后 varargs 参数位置；Java 8 重编译的最小不可见注解对照保留全部属性值。
- [x] 2.4 对个数不符、重复类型、非法名、不可拼写元素及损坏/预算/取消作明确拒绝或真实 stop；验证不输出半条注解、不把其它位置注解吞并，也不改类级注解及方法 IR。

## 3. 三方执行与主代理验收

- [x] 3.1 用修后完整 Engine/CLI 原样重放 1.1/1.2。1.1 的原/JADX/Jarde 完整类源码均 `javac --release 8` 与 `java -Xverify:all`，四行反射逐项同值；1.2 验证原/JADX 可执行、Jarde 文本/JSON 与受控拒绝来源，并用可编译最小夹具比较重编译的运行时不可见属性。1.2 完整 Jarde 类的既有 `wideAndVarargs` 方法体仍无法编译，已用冻结前/修后 CLI 确认失败相同，未手改生成源码；范围和输出见 `member-annotation-uses/post-change-replay/verification.md`。
- [x] 3.2 root 独立审查成员壳归属、参数位置/slot、原子拒绝、文本/JSON、预算/取消；以最终 CLI `19ede7a6…d0552` 在独立复制目录重放主样本与边界，另重放不可见属性最小类。reader 160/160、member reader 7/7、class-source 33/33、CLI 16/16、annotation-default 8/8、预算/取消 2/2、historical 1/1、fingerprint 5/5、fmt、Clippy 与 OpenSpec strict 66/66 均按 `verification.md` 通过；既有 Clippy type-complexity 与 bulk pin drift 独立保留。
