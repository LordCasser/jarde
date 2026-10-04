## Why

[桥隐藏裸父类头擦除派发错值实证](../../evidence/java-syntax-2026-10-04/bridge-superclass-rawheader-misdispatch/README.md)：已验收切片 `recover-bridge-admission-gates`（合并 5f07e13c，CI 绿）隐藏参数收窄桥时，其消隐前置只覆盖**接口边**（`facade.rs:29873-29875` 的 `use_kind == InvokeInterface && parameter_cast_form`），**父类边不设前置**——理由是注释 29870-29872 的"A *superclass* contract … the raw header degrades the narrowed override into an ordinary overload, **which still compiles**"。但 root 用真实二进制端到端复现证明：**"可编译" ≠ "行为一致"**。

单变量判别（[fixture/](../../evidence/java-syntax-2026-10-04/bridge-superclass-rawheader-misdispatch/fixture/)）：

| fixture | 源码父类 | jarde 渲染类头 | javac 重编再生桥数 | 擦除引用 `b.set("x")` | 判定 |
| --- | --- | --- | --- | --- | --- |
| `Specialized` | `extends Box<String>`（顶层，名无 `$`） | `extends Box<java.lang.String>`（参数化） | 2 | `SPEC.set(String) ran` ✓ | 安全 |
| `Spec` | `extends Outer.Box<String>`（嵌套，名 `Outer$Box` 含 `$`） | `extends Outer$Box`（**裸**） | 1（仅协变 `get`） | **`BOX.set(Object) ran`** ✗ | **静默错值** |

根因链（三段，各已读码/实测确认）：(1) 父类参数化投影 `class_source.rs:6444`（`parent.binary_name.contains(&b'$')`）拒绝含 `$` 的父类名 → 嵌套泛型父类退裸头；(2) 桥准入前置（`facade.rs` 的接口边分支，锚点 `*use_kind == ReferenceUse::InvokeInterface && parameter_cast_form`，现约 31185–31188；**行号已漂移**——本片原写 29874，因本片自身新增代码与相邻片而位移，以锚点名为准）只问接口边，父类边不问；(3) 裸头下 `set(String)` 对继承的 `set(Object)` 是**重载**非**覆写** → javac 不再生参数桥（实测 ACC_BRIDGE=1）→ 经父类型擦除引用的 `set` 虚派发路由到父类体。协变返回桥（`get`）不受影响：同擦除签名 `()Object` 仍是覆写，裸头下 javac 仍再生（`SPEC.get ran` 两侧一致）。

影响面：真实代码中**嵌套泛型父类的参数特化子类**（`class Repo extends Base.Dao<User>`、`class IntList extends AbstractList<Integer>`，binary 父名含 `$`）——桥隐藏但父类头退裸 → 经父类型擦除引用的参数化方法（`set`/`add`）静默路由到父类实现。注意 `BR$StrBox`（已验收片正例）恰是此形，但其 `Box.set/get` 体为空/null **掩盖**了错值，故该片的三方行为验收（`s`/`0`）未暴露——本 bug 需**可观察**父类体才显现。

## What Changes

- **桥消隐前置从"仅接口边"扩到"接口边 ∪ 父类边"**：参数收窄桥（`parameter_cast_form`）的契约属主为父类边（`InvokeVirtual`）、且投影后类头对该擦除父类**未带类型实参**（裸形 `extends Outer$Box`）时，**拒绝隐藏该桥、保持桥可见**（响亮、可编译、行为正确），与接口边既有前置对称。
- 协变返回桥（`bridge_return == target_return` 且非 parameter_cast_form，或 `bridge_return == Object` 的既有协变形）**不受此门影响**——它们裸头下仍正确覆写、javac 仍再生，隐藏安全（`Specialized.get`/`Spec.get` 实测一致）。
- **不改**父类参数化投影的 `$` 拒绝（`class_source.rs:6444`）——那是 `recover-parameterized-superclass-nested-headers`（**待立项**，root 2026-10-04 核实该目录尚不存在）的根治域；本片只堵桥隐藏的行为洞（拒绝隐藏），使该形退回"桥可见 + 裸头"的可编译且行为正确态。
- `BR$StrBox` 等父类体不可观察的形：桥从隐藏变为可见，`javac` 仍通过（桥声明合法，见 bridge-method-patrol README 的参数-cast-是合法重载结论），行为不变（其 set/get 空体两侧一致）——但呈现多出桥成员声明，须在 corpus 双腿扫描如实记录该差异。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：参数收窄桥的消隐以"契约属主的类型实参在类头可见"为前置，接口边与父类边一致适用；裸父类头下保持桥可见，避免擦除派发静默路由到父类体。

## Impact

`src/facade.rs`（桥准入前置：新增父类边对称判定，与 `bridge_interface_contract_generic`（30028）并列一个 `bridge_superclass_contract_generic`，读同一类 `Signature` 事实；扩展 29873-29898 的守卫块纳入 `InvokeVirtual` 边）；测试（`tests/class_source.rs` 补裸父头参数收窄桥负例 + 协变返回正例零回退）。**无新机制、无 reader 改动**（类 `Signature` 与物理 `super_class` 均已在准入段可得）。`recover-bridge-admission-gates` 已交付的接口边前置、门 1/门 2、`negative/`/`orphan/` 负例零回退。
