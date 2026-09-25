# 根代理验收：Java 8 lambda 描述符适配

## 结果与实现边界

同一 `LambdaMetafactory` 调用点的擦除 SAM 参数、instantiated 参数、implementation 参数现在分别留在 `lambda::Plan`。`Builder` 用擦除类型声明 lambda 参数，在函数调用时先做已证明的动态 `Object → String/String[]` 检查，再用实现参数类型固定目标重载。无需参数适配时仍保留方法引用；无捕获实例接收者、构造器按实现 handle 的输入次序写入。返回只按 implementation → 擦除 SAM 的实际转换处理，instantiated 返回不会凭空在函数体插入检查。bound receiver 的创建阶段 null 失败无证明时拒绝改写。本项未新增泛型解析器、继承解析器或新的表达式种类。

这比直接照搬 JADX 的 `sameArgs → useRef` 决策更严格：当前 Jarde 类声明仍可为 raw SAM，`String` 动态检查不能委托给泛型目标编译时推断。JADX 代码位置与算法取舍见 [design](design.md)。

## 完整类与边界证据

永久 class `tests/fixtures/p3-lambda-adaptation/v8/LambdaAdaptationProbe.class` 为 Java 8 major 52、2205 bytes，SHA-256 `1f76c28361a2c54405e82950812863da1ea2f5a1012ef3a07e1049e5e4d45c6f`。root 将**未裁剪**的原 class、安装的 JADX 1.5.6 输出、当前 Jarde 输出分别作为完整类 `javac --release 8` 编译并 `java -Xverify:all` 执行；三者 27 行完全一致，覆盖 String/Object 重载、数组类型检查、String→实现 Object 仍需先 CCE、无捕获 receiver/constructor 的异常优先级与调用计数、primitive 同型和 raw/typed `Supplier` 返回。Jarde 的永久 JDK 回归又独立比较了原 class 与当前完整类的 27 行输出。

`jarde-java --lib` 149/149；`p3_java_recovery` 32/32；`p3_lambda_adaptation` 11/11（含 2 项显式 JDK）；`p3_invocation_arguments` 4/4（含 JDK）；immediate functional receiver 2/2；reference cast 6/6、deferred value order 3/3（均含显式 JDK）；语料指纹 5/5，1 项仅供显式重录的测试保持 ignored。`p3_java_recovery` 的旧手工 bound-null 断言原来期待不可编译的 `Object local1::run`，现在按真实 frame/site/implementation 捕获类型不一致明确拒绝。捕获正例的完整类与原 class 均由永久 JDK 测试按 `--release 8` 重编并在 `-Xverify:all` 下执行，四行结果相同；bound-null 那一行使用原 class 对照，不冒充 Jarde 对拒绝形状的恢复。

`cargo fmt --all -- --check`、`openspec validate preserve-lambda-descriptor-adaptation --strict` 通过。严格 `cargo clippy -p jarde-java --lib -- -D warnings` 仍被共享工作树 15 项告警阻断，分布在 enum/region/report/build/reuse；新 `parse_method_input` 返回类型和本轮值渲染 fallback 泛型参数的 32 项新增告警已定点消除。不能称严格 Clippy 已通过，后续按独立 lint 清理任务收口。永久语料 fingerprint 对 582 文件通过；reader 的人口钉值仍为旧 `(164,1152,98,381,8)`，root 实测本工作树为 `(216,1375,128,613,8)`，属于另列的 census 更新任务，不借本语法修复改全局断言。

## 尚待闭合的来源与资源契约

[预算与来源实测](verification-budget.md)已证明 essential/all 正文一致，零捕获动态 cast 指向真实调用点 BCI 0/CP #7，而非伪造的 `checkcast`；带捕获适配则将 site BCI 5/CP #13 与真实捕获装载 BCI 4 分开。对两个正例的 Builder 内部 Local/Cast IR 批次和 bound-null 拒绝控制的计划工作都以 site 定位的低预算独立停下，没有把停止转换成普通 fallback，也没有发布半个适配正文。

直接方法恢复在预先取消时交付空正文、0 read/IR usage，却报告 `jre_ir_table_missing` 而非 `Cancelled`；它属于已有入口停止分类问题。直接进入 lambda 计划的预取消已精确返回 `Cancelled { at: 42 }`，不把上游误分类写成本轮的通过或失败。
