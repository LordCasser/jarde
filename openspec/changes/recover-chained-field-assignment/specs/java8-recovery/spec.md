## ADDED Requirements

### Requirement: 链式字段赋值 SHALL 呈现为按求值序的独立赋值组

当一次求值的 dup 值跨**恰好 n 个 putfield** 存活、且其全部消费都是这些 putfield 时，系统 SHALL 把该链呈现为 n 个独立字段赋值，序与字节码序一致（源从右到左求值序）——右值求值**一次**的语义由呈现保持：常量/纯表达式直书各赋值，**含副作用的表达式**呈现为 temp 变量形（`T tmp = <expr>; f1 = tmp; …`）。

局部链（dup 跨 istore）的既有呈现 SHALL 逐字不变。dup 值存在**非 putfield 消费**（表达式内使用如 `a = (b = 5) + 1`、混合消费）时 SHALL 保持既有拒绝。

#### Scenario: 常量链恢复

- **WHEN** `static void chain(){ CH.a = CH.b = CH.c = 5; }` 经 `class-source` 呈现
- **THEN** 恢复为按序独立赋值（`CH.c = 5; CH.b = 5; CH.a = 5;`）、0 引注；整类渲染源集 `javac --release 8` exit 0、`main` 输出与原 class 逐行一致（a/b/c 均 5）

#### Scenario: 副作用右值求值一次

- **WHEN** 链右值含方法调用（合成探针 `a = b = gen();`）
- **THEN** 呈现为 temp 形（gen() 调用恰好一次）、行为与原 class 一致

#### Scenario: 局部链与表达式内消费

- **WHEN** `x = y = 7`（局部链）经呈现——既有拆分逐字不变；`a = (b = 5) + 1`（表达式内消费）经呈现——保持既有拒绝

#### Scenario: 零回退

- **WHEN** 单字段赋值与局部链既有测试族经运行
- **THEN** 全部逐字通过

### Requirement: 字段读改写的 receiver 副本 SHALL 呈现为读与写的复合赋值形

当一次 `dup`/`dup_x1` 的两个副本分别是一成员**读**（getfield）与**写**（putfield）的 receiver、读的值是写值的计算依赖（同一基本块、读紧接副本之后）、且 receiver 源可被文本重写（常量、局部读、及其上的纯运算）时，系统 SHALL 把该读改写呈现为一条赋值语句 `receiver.field = receiver.field <op> value;`——`+=`/`-=` 的既有 int 形仍由既有更新规则呈现，`String` 字段的 SB 链形（`field += "…" + x`，javac 发 `dup_x1`）SHALL 把该链折回 `+` 呈现。receiver 源不可重写（调用结果等）、读与写不是同一成员、或副本存在其它消费时 SHALL 保持既有拒绝。

#### Scenario: String 累积字段形恢复

- **WHEN** `String field; SC add(String x){ field += "[" + x + "]"; return this; }` 经 `class-source` 呈现
- **THEN** 恢复为 `this.field = this.field + "[" + x + "]";`、整方法 0 引注；`new SC("f").add("x").add("y").field` 的可编译错面（`f` vs `f[x][y]`）关闭，整类编译运行输出与原 class 一致

#### Scenario: 位运算复合恢复

- **WHEN** `int flags; void enable(int b){ flags |= 1 << b; }` / `void disable(int b){ flags &= ~(1 << b); }` 经呈现
- **THEN** 恢复为 `this.flags = this.flags | 1 << b;` / `this.flags = this.flags & (1 << b ^ -1);`、0 引注，行为与原 class 一致

#### Scenario: 不可重写 receiver 与异成员保持拒绝

- **WHEN** receiver 副本的源是调用（`holder().ia |= 1`）或读/写成员不同经呈现
- **THEN** 保持既有拒绝（整方法引注），不呈现任何赋值
