## Why

[非静态折叠巡查](../../evidence/java-syntax-2026-10-03/inner-class-folding-patrol/README.md) 与 mixed 片披露共同锁定第二层 MVP：非静态成员类（`class Inner`）不折叠——构造以合成首参透传呈现（`new N1$Inner(this, arg1)`）、`this$0` 字段声明可见、javac 对私有捕获生成的 `access$NNN` 桥完整呈现（源码无此成员）、限定 `outer.new Inner(9)` 无证明（既有缺口，分离/折叠两态同错）。非静态内部类是 Java 命名习惯的核心形态（Builder/Listener/迭代器），折叠后四类合成物消隐才能交付真实源码形状。

## What Changes

- **折叠**：非静态成员类按嵌套声明折叠（`class Inner { … }`，与 mixed 片静态子集共存的装配合并）。
- **this$0 消隐**：ctor 首参（synthetic `this$0` 位）与字段声明在折叠呈现中隐藏；构造调用位按限定语法呈现——限定符为外围 `this`（或外围语境可推断）→ `new Inner(args)`；显式限定 → `qualifier.new Inner(args)`。
- **access$NNN 消桥**：折叠族内 javac 合成静态访问桥（`access$000` 等）在呈现中隐藏，其调用位按桥体直译重写（`access$000(o)` → `o.field` 读写限定形）。
- N1 家族（含 `outer.new Inner(9)` 路径）整家族集重编行为一致（`10/7/13`）；分离呈现（逐类输出）与既有全部折叠通道零回退。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：非静态成员类按嵌套声明折叠，this$0/access 桥消隐，构造按限定语法呈现。

## Impact

`src/member_inner.rs`（非静态行折叠准入）与 `src/facade.rs`（装配合并）、呈现层（ctor 消参/限定 new/桥消隐与重写）及测试；复用两片折叠机械 + synthetic-ctor 片的合成序知识。分离呈现与既有通道零回退。
