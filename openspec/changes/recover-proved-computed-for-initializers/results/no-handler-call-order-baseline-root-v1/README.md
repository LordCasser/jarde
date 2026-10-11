# 无 handler 的独立调用顺序基线

这是当前产品 `74b02d205c6b99f223c5fe2dbc5095d370825972` 的 while 基线，不是尚未应用的 computed-for 候选验收。冻结 CLI2 SHA `8e70234797af43e114e94cd08ba0a8f120d59d0e9c638a20a8cf64e9876fd64b`，完整4695输入 build/clean 证据见 [CF12 final CLI](../../../project-integer-constant-names-with-refused-member-family/results/full-workspace-final-cli-root-v1/final-cli-build/execution.json)。

root 实际运行36命令、双JDK8/23、原与合法 pending class × default/all 八份完整 Jarde 源码。完整源码字节等于 CLI text，原样 javac 重编；12个运行侧共60行结果完全一致。f/g顺序、两次调用的各自异常位置和 end 修改均核对。find 无 handler，induction local6 与参数不复用；patcher 仅交换相邻 store6/g 两条指令，Code length、后续 BCI/stack map 不改。所有原/pending及重编类通过 `-Xverify:all`。

原始字节次序 f()-1;store;g 对应 `local6 = f() - 1; g(); while`。pending f()-1;g;store 则对应 `int saved0 = f(); g(); local6 = saved0 - 1; while`。现有 prepare_deferred_bindings 会在独立 g 边界前物化 f。这否定了此前静态推测的“必然在 g 后求值 f”，不需要据此添加 purity walker；新的 for 必须在实际候选中再次核验物化或准确拒绝。

root 独立 verifier 实际 exit0，并补核准确 JDK tool、原源码/八个生成源码 compile argv 与12个 runtime 行均绑定 execution 中的同一命令；所有19条物理 BCI、report 的 BLAKE3 owner identity、UTF8来源 span、各方法 default/all text/map 恒同。SHA-256 用于实际 class 文件 pin，未冒称是 report digest。16个生成 class 的 hash 是**运行后观察**，没有运行前全量 hash 的声明。Jarde classpath 第一项有实际新 class，第二项只提供固定 Runner harness。

本次新增控制没有执行 JADX；此前 [完整三方控制](../complete-controls-root-v1/README.md) 已记录普通 javac 初始化被 JADX 跨 g 提升的真实错误。后续候选完整对照不得借用本次 while 结果。

`actual-v2/` 保存全部执行、raw、输入/生成源码和class；`source/` 是本次已运行控制与patcher；`verification/` 保存已执行 runner、独立/root补充验收及stdout/stderr。`preserved-failed-v1/` 保留实际5命令失败：第一次漏传 single-class policy，CLI在第五命令准确返回环境policy不匹配，不能把此失败当恢复失败或成功运行。archive-inventory列出复制前后字节相同的原路径/相对路径/pins。Rust target 未产生。
