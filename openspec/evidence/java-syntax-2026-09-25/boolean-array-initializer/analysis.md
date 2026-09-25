# `new boolean[]{raw int}` 的数组初始化器折叠审计

日期：2026-09-25。此目录记录独立、只读语法审计。除新增本证据目录外，没有修改生产代码、测试、现有 change 或 roadmap。

## 结论

验证通过的 Java 8 class 可以由 `new boolean[]{raw int...}` 形状的字节码构成：普通 Java 源码先生成 `int[]`，然后对方法返回 descriptor、`newarray` atype 和 `iastore` opcode 作等长、逐项限定的补丁。JVM 能按 `bastore` 语义执行并把任意 int 的最低位读回为 boolean。当前 Jarde 的已证明数组初始化器折叠遇到这种 class 时，会在元素准入处拒绝；JADX 1.5.6 输出非法 Java 初始化器，`javac` 对每个 int 元素报 int 不能转换成 boolean。

这和普通 `bastore` 写语句是独立消费位置。`recover-boolean-array-stores/design.md` 明确列出“不折叠数组初始化器”为 Non-Goal，所以建议在普通写入工作完成后另行提案，避免把两种呈现流程塞进当前 change。未来实现应限于 `ArrayInitializers::prove` 已完整证明的组件为 `boolean`、真实 opcode 为 `bastore` 的计划，不新增通用 pass；在该证明中为每个元素保留显式有序 `(ValueId, store_bci)` 配对，让低位转换节点以真实 store BCI 作为派生来源。

## 最小来源与折叠路径

`BoolInit.java` 仅含普通 `int[]` 返回与字面量初始化：

```java
final class BoolInit {
    static int[] values() {
        return new int[] {0, 1, 2, 3, -1};
    }
}
```

`patch_class.py` 在常量池把 `()[I` 改为 `()[Z`，在 `values` 的 Code 中把唯一 `newarray` 的 atype `10`（int）改为 `4`（boolean），并把五条 `iastore` (`0x4f`) 改为五条 `bastore` (`0x54`)。没有修改 Code 长度或其它字节。原 class SHA-256 是 `5304fa21e9594a0ab6e04e0f79ee0da3bbcdcbc85e3019183d7cdd5e7b22da7d`；patch class SHA-256 是 `e0e8cf5cf99fdb3e402db16e6d0b86db1b8b8672df2e684523c72a67df724a3f`。两者均为 189 bytes、classfile major version 52。完整 descriptor、atype、opcode 偏移和 Code hex 见 `reproduced/patch.json`，逐项反汇编见 `reproduced/logs/javap-{original,patched}.json`。

当前 `prove_array_initializer` 在每一轮已经同时掌握 `stored_value` 与真实 `store.bci()`，并验证数组组件事实与 store opcode 一致，然后将元素值依序放入 `ArrayInitializer.elements`。`ArrayInitializer.owned` 也会收集 dup、index 与每条 store 的 BCI；因此 closure 证明覆盖了具体 store。不过结构中 `elements: Vec<ValueId>` 与 `owned: Vec<u32>` 是分开的，`owned` 还包含分配长度和其它 scaffold BCI，没有稳定的 element→store 对。

`render_value` 折叠初始化器时只迭代 `initializer.elements`，按 `self.render_value(value, at, ...)` 渲染，并将同一个当前消费点 `at` 传给 `array_initializer_element`。本例的 consumer 是 BCI 23 `areturn`，元素真实 stores 则是 BCI 6、10、14、18、22。由此，当前拒绝文本和 AST 来源 primary 指向 BCI 23，而不是任一真实 `bastore`。如未来在此加入低位表达式，不应重用 consumer BCI；最小数据结构变化是在已证明计划中保留顺序配对或同等明确的 store BCI 列表，并以该 BCI 产生转换派生来源。这样仍由现有闭包证明、来源及预算框架负责，不需要通用的数组 store 重建 pass。

## 原 class、JVM、JADX 与 Jarde 结果

`reproduce.py` 用 `javac --release 8 -g:none` 编译原始 source-only int[] 样本，再调用 patcher，执行 `javap`、runner、JADX 和 Jarde。复放工具版本：OpenJDK/Javac 23.0.1、JADX 1.5.6、Jarde CLI 0.1.0。归档复放的 Jarde CLI SHA-256 见 `reproduced/summary.json`（`a651e4ab31f512b794277c05ec788d74122a7a7c8495e8d67d601d0c9e28dd00`）。

原 class 在 `java -Xverify:all` 下执行并读回 `[0, 1, 2, 3, -1]`；patch class 同样以 `-Xverify:all` 成功执行，读回 `[false, true, false, true, true]`。这确认该 patched class verifier-valid，且本样本真实运行包含偶数、奇数及负数，不是按 source 语法臆造出来的 class。

JADX 阶段退出 0，输出 `return new boolean[]{0, 1, 2, 3, -1};`；以 Java 8 重编在这五个 initializer 元素各报一次 int→boolean 错误，状态 1。其本地 1.5.6 算法在 `TypeUpdate.java:609-635` 的 `arrayPutListener` 将组件类型推给数组写入值；`InsnGen.java:470-476` 的 APUT 生成逻辑直接写 `array[index] = value`。初始化器输出保留了相同的元素类型冲突，没有编码 JVM boolean `bastore` 的最低位规则。

Jarde `class-source --policy single-class --evidence all` 进程状态为 0，但方法正文只有拒绝 marker，不能视作恢复成功。它在 `values()[Z` 的 consumer BCI 23 输出 `the array initializer element at BCI 23 has no boolean evidence, so its int spelling cannot be assigned to boolean`，并列出创建、所有五个 store 及 return 的相关 bytecode 来源。关键阶段原样输出保存在 `reproduced/logs/jarde.json`；JADX 源与 javac 错误分别保存在 `reproduced/jadx/sources/defpackage/BoolInit.java` 和 `reproduced/logs/javac-jadx.json`。

## 复放与产物

在仓库根目录运行：

```sh
CARGO_TARGET_DIR=/tmp/jarde-bool-array-init-audit-target cargo build --locked -p jarde-cli
python3 openspec/evidence/java-syntax-2026-09-25/boolean-array-initializer/reproduce.py \
  /tmp/jarde-bool-array-init-audit-target/debug/jarde-cli /opt/homebrew/bin/jadx
CARGO_TARGET_DIR=/tmp/jarde-bool-array-init-audit-target cargo clean
```

上述复放已完成；私有 Cargo target 已清理（移除 2702 个文件、约 1.0 GiB）。永久审计输出仅在本目录 `reproduced/`，当前占用约 136 KiB。`summary.json` 汇总阶段和哈希，`logs/` 保留命令、exit status、stdout/stderr；原 class、patched class、runner class 和 JADX 源码均保存在该 `reproduced/` 下。所有产物均为本次只读审计证据，没有进入测试 fixture 或 OpenSpec change。
