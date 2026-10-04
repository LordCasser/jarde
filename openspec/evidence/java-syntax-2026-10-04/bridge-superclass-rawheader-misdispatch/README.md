# 桥隐藏的裸父类头擦除派发静默错值（2026-10-04，root 验收 ncl 时旁证发现）

**性质：已验收切片 `recover-bridge-admission-gates`（合并 5f07e13c，CI 绿）中的一个潜伏静默偏离**，由 root 在验收 `recover-nested-class-literal-values` 期间，把"池形类型名的结构反射陷阱"不变量推广到桥隐藏场景时旁证发现。root 不凭推断——用真实 main 二进制（含已合并 bridge 片）端到端复现。

## 单变量判别（决定性）

两个 fixture 只差父类名是否含 `$`（即嵌套 vs 顶层），桥形态、擦除调用方式全同：

| fixture | 源码父类 | jarde 渲染的类头 | javac 重编后再生桥数 | 擦除引用 `b.set("x")` | 判定 |
| --- | --- | --- | --- | --- | --- |
| `Specialized`（[fixture/Specialized.java](fixture/Specialized.java)） | `extends Box<String>`（顶层，binary 名无 `$`） | `extends Box<java.lang.String>`（**参数化**） | 2（两桥都再生） | `SPEC.set(String) ran` ✓ | **安全** |
| `Spec`（[fixture/Spec.java](fixture/Spec.java)） | `extends Outer.Box<String>`（嵌套，binary 名 `Outer$Box` 含 `$`） | `extends Outer$Box`（**裸**） | **1**（只再生协变 `get` 桥，不再生 `set` 参数桥） | `BOX.set(Object) ran` ✗ | **静默错值** |

原类基线（[results/original-nested.out](results/original-nested.out)）：`SPEC.set(String) ran` / `SPEC.get ran`；渲染 Spec 重编后（[results/rendered-nested-misdispatch.out](results/rendered-nested-misdispatch.out)）：`BOX.set(Object) ran` / `SPEC.get ran`。

## 根因（三段链，每段都已读码/实测确认）

1. **父类参数化投影拒绝含 `$` 的父类名**：`src/class_source.rs:6432` 的 `direct_parent_candidate` 判据含 `parent.binary_name.contains(&b'$') → Err`，故 `Spec extends Outer.Box<String>` 的类头**退回裸形** `extends Outer$Box`（丢失 `<String>`）。这与 `recover-parameterized-interface-headers`（本片同期立、待实施）要解决的接口裸头同源，只是发生在父类边。
2. **桥准入的消隐前置只覆盖接口边**：`src/facade.rs:29873-29875` 的守卫是 `if … use_kind == ReferenceUse::InvokeInterface && parameter_cast_form`——**父类边（`InvokeVirtual`）不设前置**。其注释（29870-29872）明写理由："A *superclass* contract is different: the raw header degrades the narrowed override into an ordinary overload, **which still compiles**, which is why superclass edges are not questioned here."
3. **"still compiles" ≠ "行为一致"**：裸父类头下，`Spec.set(String)` 对继承来的 `Box.set(Object)` 是**重载**（不同参数类型）而非**覆写**，故 javac 不为它生成参数桥（实测 `Spec.class` 重编后 ACC_BRIDGE=1，仅协变 `get` 的桥）。经 `Outer.Box` 擦除引用调用 `set` 时，虚派发解析到 `Box.set(Object)`（父类体）而非 `Spec.set(String)`（子类体）——**静默错值**。

**为何协变返回桥（`get`）不受影响**：`Spec.get()` 返回 `String`、擦除签名 `()Object`，对继承来的 `Box.get()()Object` 是**同签名覆写**，裸头下 javac 仍为它生成协变桥（实测再生），故擦除调用正确路由到 `Spec.get`（`SPEC.get ran` 两侧一致）。危险面**仅参数收窄桥（`set`）∧ 裸父类头**。

## 与已闭合事故的同族关系

这与本会话已处置的三个"可编译且行为不同"事故同族，都是"可编译性被误当作行为保真"：

- `recover-ctor-reorder-dispatch-guard`（fc868aba）：构造期虚分派下重排捕获写入 → `visibleDuringSuper` true→false。
- `recover-nested-class-literal-values` 残余边界（33575de1）：family 折叠失败回退池形 + 结构反射 → `getSimpleName` 返回池名。
- **本片（桥隐藏裸父类头）**：擦除派发路由到父类空/异体 → 子类 `set` 逻辑丢失。

## 影响面

真实代码中**嵌套泛型父类的参数特化子类**（`class Repo extends Base.Dao<User>`、`class IntList extends AbstractList<Integer>` 等，binary 父名含 `$`）：桥被隐藏但父类头退裸 → 经父类型擦除引用的 `set`/`add`/参数化方法调用静默路由到父类实现。协变返回形（`get`/`toString`/`build`）不受影响。**注意**：BR$StrBox（已验收片的正例）恰是此形——但其 `Box.set`/`get` 体为空/null，掩盖了错值，故该片的三方行为验收（`s`/`0`）未暴露；本片用**可观察**父类体（`Box.set` 打印）才使错值显现。

## 处置方向

`recover-bridge-superclass-header-precondition`（窄修复片，与 `recover-parameterized-interface-headers` 同域、应协调实施）：桥消隐前置从"仅接口边"扩到"**参数收窄桥 ∧ 父类边 ∧ 类头未参数化**"——即当参数 cast 形桥的契约属主是父类边、且投影后的类头对擦除父类**未带类型实参**（`extends Outer$Box` 裸形）时，**拒绝隐藏该桥、保持桥可见**（与接口边的既有前置对称）。协变返回桥（`bridge_return == Object` 或 `parameter_cast_form == false`）不受此门影响（它们裸头下仍正确覆写）。

根治路径是 `recover-parameterized-interface-headers` 的姊妹——让父类参数化投影也覆盖嵌套父名（放宽 `class_source.rs:6432` 的 `$` 拒绝，走同一 `InnerClasses` 行集重拼），使类头呈现 `extends Outer.Box<String>`，javac 遂重建两桥。但那属父类头投影域；**本修复片先堵桥隐藏的行为洞**（拒绝隐藏 = 响亮、可编译、行为正确），父类头投影作为后续独立片让该形也能隐藏桥。

原 class 为行为基准（`SPEC.set(String) ran` / `SPEC.get ran`）。

## 修复形状的 root 独立验证（2026-10-04，派发前用 javac 实证，非 cargo）

裁决的核心断言是"桥可见 + 裸头 = 行为正确"。root 在派发实现前先用 javac 独立验证该形状成立（避免实现者撞向错误设计）：

```java
class Box<T> { void set(T v){System.out.println("BOX.set(Object) ran");} T get(){...} }
// 模拟修复后呈现：裸父类头 + 参数收窄桥【可见】
class SpecFixed extends Box {                     // 裸头
    void set(String v){System.out.println("SPEC.set(String) ran");}
    void set(Object v){ this.set((String)v); }    // 桥可见：覆写继承的 set(Object) → 转发 set(String)
    String get(){System.out.println("SPEC.get ran"); return "S";}
}
// Drv: Box b = new SpecFixed(); b.set("x"); b.get();
```

- `javac --release 8` → **exit 0**（`set(String)` 与 `set(Object)` 是合法重载；可见的 `set(Object)` 覆写继承来的擦除 `set(Object)`）。
- 运行经 `Box` 擦除引用 `b.set("x")` → **`SPEC.set(String) ran`**（命中可见桥 → 转发），`b.get()` → **`SPEC.get ran`**——**与原类逐路径一致**。

故"桥可见 + 裸头"确实修复了错值（对比隐藏桥时擦除派发路由到 `BOX.set(Object)`）。协变返回桥隐藏仍安全（`Spec.get` 实测 `SPEC.get ran` 一致）——两者不对称的根因：协变返回桥裸头下仍是同擦除签名**覆写**（javac 再生），参数收窄桥裸头下降级为**重载**（javac 不再生），故前者可隐藏、后者不可。这印证了 design 决策 2 的 `parameter_cast_form` 约束面。
