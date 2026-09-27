## 1. 身份有保证的证明

- [x] 1.1 在同次 SSA 中只证明准确 `Boolean.valueOf(Z)`、`Integer.valueOf(I)`、`Character.valueOf(C)` 的 primitive 字面量、唯一直接 return 消费者及对应包装类或 `Object` 返回目标；定向测试检查 int `-128..127` 与 ASCII char 范围、错 owner/descriptor、第二消费者和不匹配返回类型。
- [x] 1.2 复用既有表达式/return 发射，在已证三种范围内输出 primitive 字面量并把调用 BCI 保留为来源；来源映射测试须覆盖 literal、调用和 return，`Byte`/`Short`/`Long` 与范围外 int/char 仍写显式 `valueOf`。

## 2. 编译运行与集成验收

- [x] 2.1 扩展 EM-25 固定回放：原/JADX/Jarde 完整类与共同 Runner 通过 `javac --release 8`、`java -Xverify:all`；Jarde 三种获证简写的类型、值和重复调用 identity 与原 class 相同，另外三种仍显式保留，`.longValue()`/null 拆箱行为不变。
- [x] 2.2 对 proof 缺失、预算及取消形态验证保留调用/物理来源或原拒绝；固定回放重跑两次并比较源码、运行与输入 SHA，确认输出稳定。
- [x] 2.3 定向 builder/class-source 测试、`cargo fmt --all -- --check`、workspace check、`git diff --check` 和 `openspec validate debox-proved-wrapper-return-boxing --strict` 均通过后勾选任务，并清理临时 Cargo target。
