## MODIFIED Requirements

### Requirement: 参数收窄桥的消隐以契约属主的类型实参在类头可见为前置

系统 SHALL 仅在参数收窄桥（`parameter_cast_form`，桥参数与源级参数不同）的契约属主类型在**投影后的类头文本中带类型实参**时隐藏该桥。契约属主为父类边（`InvokeVirtual`）而类 `Signature` 陈述父类为泛型、但投影类头因父名含 `$` 退回裸形（`extends Outer$Box`）时，SHALL 保持桥可见，不得隐藏。契约属主为接口边时沿用既有前置（`recover-bridge-admission-gates`）。协变返回桥（`parameter_cast_form == false`）SHALL 不受此前置影响——裸头下它仍是同擦除签名覆写、javac 仍再生桥，隐藏安全。系统 SHALL NOT 产出"桥已隐藏 ∧ 类头裸类型 ∧ 擦除派发路由到父类体"的可编译且行为不同文本。

#### Scenario: 嵌套泛型父类的参数收窄桥保持可见

- **WHEN** `class Spec extends Outer.Box<String> { void set(String v) {…} String get() {…} }`（`Outer.Box` 的 binary 名含 `$`、`set` 体可观察）经 jarde 渲染并家族重编
- **THEN** 类头为裸 `extends Outer$Box`、`set(Object)` 桥**可见**（转发 `set((String)arg)`）、协变 `get` 桥仍隐藏；`javac --release 8` 通过，经 `Outer.Box` 擦除引用 `set("x")` 运行输出 `SPEC.set(String) ran`（与原 class 一致），不得为 `BOX.set(Object) ran`

#### Scenario: 顶层泛型父类零回退

- **WHEN** `class Specialized extends Box<String>`（父类 binary 名无 `$`、类头投影为参数化 `extends Box<java.lang.String>`）经 jarde 渲染并重编
- **THEN** 两桥仍隐藏（javac 从参数化头再生两桥），擦除派发 `SPEC.set(String) ran`/`SPEC.get ran` 与本变更前逐字一致

#### Scenario: 协变返回桥不受前置影响

- **WHEN** 契约属主为父类边或接口边的**协变返回**桥（`String get()` 覆写 `Object get()`，`parameter_cast_form == false`），无论类头参数化与否
- **THEN** 桥仍按既有判据隐藏，擦除派发路由到子类覆写（`SPEC.get ran`），与本变更前一致
