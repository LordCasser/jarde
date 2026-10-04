## Why

`recover-anonymous-mixed-super-capture`（已验收 `e1c89d57`）与 `recover-anonymous-local-decl-site`（环 1，实施中）之后，root 实测四处 ctor-reorder fixture 的阻塞链共三环，各自被**不同**的门挡住（证据：[anonymous-chain-rings-2-3](../../evidence/java-syntax-2026-10-04/anonymous-chain-rings-2-3/README.md)）。本片是**环 3：根方法带参数形**。

冻结 fixture `tests/fixtures/proved-java-structure/anonymous-super-dispatch/` 当前被 `anonymous_super_return_type_unproved` 拒绝（root 实测渲染，拒绝文本原文："the root method return descriptor is not the exact superclass type"）。其形态（root 以 javap 逐项实测）：

```java
private static Base create(final String captured) {   // descriptor (Ljava/lang/String;)LBase;，ACC_PRIVATE|ACC_STATIC
    return new Base() {                                // 分配点在 BCI 0（直返形），唯一分配点
        void observe() { /* 读 captured */ }
    };
}
```
```text
AnonymousSuperDispatch$1:  final java.lang.String val$captured;
  <init>(Ljava/lang/String;)V:
       0: aload_0 / 1: aload_1 / 2: putfield val$captured:Ljava/lang/String;   ← 捕获写
       5: aload_0 / 6: invokespecial Base."<init>":()V                          ← super 无实参
       9: return
```

该形与已交付的两片都不同：**捕获值经由根方法的参数传入**（而非根方法内的局部），且 **super 实参集为空**（纯捕获）。既有门要求根方法描述符**恰为** `()Lparent;`，故带参形永远被拒。真实代码里"工厂方法接收参数并返回匿名子类实例"极常见，故立此片。

**本片不需要新机制**：`src/facade.rs` 的**接口匿名路径已实现同一形态**（`captured_root_parameter`，约 3490–3530），只是硬编码为 `double`（`b"D"`）。本片把该既有判据移植到父类路径，并沿用 `recover-anonymous-mixed-super-capture` 已泛化的 `ProvedCapturedParameterRead.parameter_presented`（该字段正是为跨类型复用而引入，替换了原硬编码 `Type::Double`）。

## What Changes

- `project_class_source_anonymous_super` 的根方法门放宽：**仅放宽参数表**，接受"根方法描述符 = `(捕获参数描述符)Lparent;`"，其**返回部分仍须恰为 `Lparent;`**。不得改为"忽略返回类型"——那是环 2（返回父类的超类型）的范围，本片不碰。
- 分配点的捕获实参允许是**根方法的参数**（现只接受局部引用）：参数槽须与 child 构造器的捕获角色一一对应，且该参数在根方法内恰以该分配实参形式被消费一次。
- 捕获读取的词法替换沿用既有通道，重拼目标为**根方法的参数名**（用 `class_source_single_parameter_name(root_ast, index)` 这类 AST 来源，**不得**依赖 `LocalVariableTable`——锚 fixture 以 `-g` 冻结故有 LVT，必须另冻一个 `-g:none` 同形对照证明实现不依赖调试信息）。
- 参数角色划分复用 `member_inner::partition_anonymous_val_constructor`；本片锚的划分退化为"全部 child 构造器参数都是捕获角色、super 实参集为空"。

## Capabilities

### Modified Capabilities

- `java8-recovery`：混合/纯捕获匿名类在"根方法带参数、直返分配"形下同样内联为源级 `new Base(…) { … }`，捕获读取重拼为根方法参数名。

## Impact

`src/facade.rs`（根方法门与分配点捕获实参来源）、可能 `src/member_inner.rs`（若划分需暴露参数角色给投影侧）、`crates/jarde-java/src/report.rs`（若需新增 AST 侧车读取）。**不改** `crates/jarde-java/src/emit.rs`（左端重拼属环 1）、**不改**站点扫描 `class_source_direct_return_new`（属环 1；本片锚已是直返形）。

冻结锚：`tests/fixtures/proved-java-structure/anonymous-super-dispatch/`（当前 `javac` 状态与呈现须在取证阶段实测记录，**不得**沿用旧记载）。

**顺序约束（强制）**：本片与环 1（`recover-anonymous-local-decl-site`）改**同一个门** `anonymous_super_return_type_unproved`，且环 1 在飞中。**必须等环 1 合入主线后再派发本片**，否则 rebase 冲突且验收无法定位失败原因（本会话已两次因串行改同一函数遭遇锚点漂移）。

**不得顺带做的事**：接口路径残留的 `b"D"` 硬编码泛化（`facade.rs` 约 3455/3490/3771/3800/3896 五处）——那有 `recover-proved-anonymous-local-capture`(6/6) 的已验收测试守着，须另立一片，不与父类路径混改。
