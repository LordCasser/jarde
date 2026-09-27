# DT-14 主线独立验收：枚举用户 Map 后缀

root 在 `a3d6f827` 主线源码上独立构建 CLI（SHA-256 `f25debd417ce0287d8447a98f57425cffa2fac7455027410a56973325b16cfe7`），用固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 运行 `JARDE_CLI=/tmp/jarde-root-dt14-target/debug/jarde-cli python3 openspec/evidence/java-syntax-2026-09-27/dt14-enum-init/replay.py fixed-both`。重放后的归档文件与已提交版本一致，`CustomInit` 的 Jarde 全源码 SHA-256 为 `c370b3c204171ef01475b42101170965013f9c1bfeb1b7844f1c02297db788dc`。

原 class、固定 JADX 与 Jarde 的完整 `CustomInit` 源码均经 `javac --release 8 -g:none` 编译、`java -Xverify:all` 运行，逐字节输出 `map=2:true:true`。普通 enum、literal 构造实参和已修 int ternary 的三个控制样例也三方一致；String ternary 仍在 Jarde 源码处以 `enum constant expected here` 拒绝，是 DT-11 交叉边界，不算 DT-14 全单元追平。五组 class SHA、三方出口和归档源码见 [`fixed-both/results.json`](fixed-both/results.json)。

生产证明仅接收完整的两常量 enum 前缀与 27 条准确后缀指令；字段身份、`HashMap` 构造、`values()` 数组、循环 backedge、`name()`/`Map.put` 调用及同次结构候选逐项核对，之后复用现有 AST statement emitter 投影静态块。额外 Map 写、额外调用、异常边、其他数组来源和改动 `iinc` 的测试均保持物理常量字段及 `<clinit>`，不发布半份 enum 投影；预算停止和取消也不发布部分结果。物理 `<clinit>` 的 source map 对 BCI 39/53/68 仍可定位 Map 写、循环和 `put`。

独立检查通过：`cargo test -p jarde --lib --locked`（145 项）、`cargo test -p jarde-java --lib --locked`（233 项）、`cargo check --workspace --locked`、`cargo fmt --all -- --check` 以及 `openspec validate recover-enum-user-initializer-suffix --strict`。本次只验收该固定后缀形状；其他 enum 用户初始化仍按各自 class/Code 证据扩验。
