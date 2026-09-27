# DT-29 P3：四段跨块条件拼接

`replay.py` 使用独立 `BranchedBits`/`A`/`B` 完整类族和同一反射 Runner，分别从原始、固定 JADX、Jarde 的全部物理类以 `javac --release 8 -g:none` 重编，再执行 `java -Xverify:all`。Runner 用 B 实例调用 `bits(A)`；三方输出均为 `1111`、`0000`、`1010`、`0101`。脚本校验固定 JADX revision 与 `TestFieldCast.java` SHA-256；结果保存在 `results.json`。运行前用专用 `CARGO_TARGET_DIR` 构建 `jarde-cli`，并以 `JARDE_CLI` 指向该二进制。

固定 `FieldCast.bits(A)` 的物理 BCI 是：`new/dup/<init>` 0/3/4；四次布尔读取 8/25/42/59；四次 `append(String)` 21/38/55/72；`toString` 75；`areturn` 78。前三次读取是同一 A owner 的 `getfield Z`，最后一次是 A 的静态 `(LA;)Z` getter（Java 8 对嵌套 private 字段使用 `access$000`）。证书逐段核对菱形 CFG 的正常边、两臂 String 常量的 SSA 定义、join 栈 Phi 与唯一 append 消费，并核对同一 builder 的上一追加返回值、无别名、无异常 handler、完整 33 条指令和最终返回。字段名与 String 字面量均取自物理事实，不是固定文本。独立 alternate 夹具以其它字段名、`Y/N` 常量和 `readDelta` getter 验证这一点；getter 把前三字段清零且计数，Runner 的 `YYYY:1:false:false:false` 同时观测其恰好执行一次并在前三次读取之后执行。原/JADX/Jarde 三方一致。全部 33 个固定 BCI 均映射到源码。第 4 个 getter 的目标调用由物理 CP owner/name/descriptor 确定；其私有字段转发由 A 物理类的 accessor 恢复单独核对，P3 不从方法名推断字段。

原 `prepare_carried_conditional_pair` 将“前一段 join 与下一段 test 同块”误当作双参数调用；在此字节码中 BCI 21 的单参 `append(String)` 已独立消费前一 Phi，BCI 38 是下一次 append。现在只有这项前置消费的 SSA/调用身份成立时跳过双参数配对；真正双参数调用及其失败路径仍由 `p3_carried_conditional_arguments` 测试覆盖。

`BranchedBitsNegatives.java` 的 alias、reused、exchanged、effect、handler、overload、missing 七个方法分别破坏 builder 唯一性、Phi 单次消费、条件值顺序、独立效果、异常边、追加重载和第四次读取；Jarde 都保持带 `@bytecode` 的拒绝。预算与取消通过 Rust 定向测试验证没有部分源码。`jre_concat_split` 的原同块入口与一般跨块拒绝保持有效。本证书只覆盖四段、33 指令、一个 builder 的完整方法；以后扩展其它段数或独立语句时，需要把执行位置和别名证明推广为独立的有界链分析，不能仅放宽段数检查。
