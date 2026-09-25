## 1. 冻结声明与拒绝证据

- [x] 1.1 重放 `method-local-generic-throws` 的 `-g`/`-g:none` 原/JADX/Jarde 对照，固定方法 `Signature`/`Exceptions`、原类验证执行、泛型反射、类与覆写调用方的 Java 8 重编结果及哈希；以 `replay.sh` 退出码 0 和 `analysis.md` 验证。
- [x] 1.2 增补无正文 `<T extends Number> T echo(T)` 的参数/返回正例及无 Signature `throws` 后缀却有物理 `Exceptions` 的正例；三方重放类、覆写/强类型调用方及泛型反射，保存来源与哈希。证据：`no-body-generic-methods/replay.py` 退出码 0、`analysis.md`。
- [x] 1.3 构造 verifier-valid 的未绑定变量、异常擦除不符及未知父类覆写契约负例；逐例核对 `java -Xverify:all`、reader/源码局部拒绝与物理声明可重编，并留可重放命令。

## 2. 无正文方法泛型候选

- [x] 2.1 在现有方法 Signature 接缝让无正文方法进入 reader 已有作用域/擦除证明，并仅对物理父类为 Object、无接口、方法名不覆盖 Object、且无未证明同类调用的顶层抽象类或根接口成员准入；用正常类/接口和继承/同类调用负例的定向测试验收。
- [x] 2.2 复用结构化类型拼写器和已证类/方法变量作用域，完整拼写方法类型参数及可支持的参数/返回/异常位置；`throws` 变量检查确定 JDK 异常根，空后缀保留物理异常。用 1.1/1.2 及根接口 fixture 的完整类、覆写/实现类与调用方 `javac --release 8`、`java -Xverify:all` 及泛型反射验收。
- [x] 2.3 复核注解/varargs/异常混合顺序、方法来源、局部拒绝、低预算/取消及 essential/all 正文一致性；运行定向 Rust 测试、格式及 `git diff --check` 验收，不修改有正文方法投影。

## 3. 独立验收

- [x] 3.1 root 审读无继承契约证明、方法局部作用域、异常上界、原子发布及停止路径；独立重放正反例，运行 reader/query、类源码与泛型相邻回归、适当 Clippy、`cargo fmt --all -- --check` 和 `openspec validate recover-no-body-generic-method-signatures --strict`，记录结果并清理私有 Cargo target。见 [独立验收](verification-root.md)。
