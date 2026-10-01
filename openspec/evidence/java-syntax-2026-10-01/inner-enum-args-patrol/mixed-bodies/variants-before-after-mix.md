# 变体前后记录：组合形态负例/变体（任务 1.2）

四个 verifier 有效负例/变体，源与 jar 冻结于本目录（[variants-mix.jar](variants-mix.jar)、[fam-broken-forward-mix.jar](fam-broken-forward-mix.jar)，SHA 见 [sha256-mix.txt](sha256-mix.txt)）。每方 `java -Xverify:all` 通过后才记录；前 = 主线 `03552a2e`，后 = 本 change 实现后。源转录：[Trio-mix.java](Trio-mix.java)、[Gs-mix.java](Gs-mix.java)/[Simple-mix.java](Simple-mix.java)、[Holder-mix.java](Holder-mix.java)（`javac --release 8 -g:none`）。

| # | 变体 | 形状 | 前（主线） | 后（本 change） |
| --- | --- | --- | --- | --- |
| 1 | 嵌套+混合叠加一形 | `p.Holder$Op`：`PLUS(1){…} MUL(2){…} ID(0)` | 逐字段（[holder.base-mix.java](holder.base-mix.java)） | **折叠** `PLUS(1) { … }, MUL(2) { … }, ID(0);`（[holder-mix.jarde.java](holder-mix.jarde.java)），重编 `java -Xverify:all` 运行 `7`/`12`/`0` 与原 class 一致 |
| 2 | 三参带体 | `p.Trio`：`ONE(1,"one",(byte)4){…} TWO(2,"two",(byte)5){…}`，ctor `(String,int,String,byte)` | 逐字段（[trio.base-mix.java](trio.base-mix.java)） | **折叠** `ONE(1, "one", (byte) 4) { … }, TWO(2, "two", (byte) 5) { … }` + `private Trio(int arg0, java.lang.String arg1, byte arg2)`（[trio-mix.jarde.java](trio-mix.jarde.java)），重编运行输出逐行一致（`ONE:10 0`/`TWO:20 1`/`ONE TWO`） |
| 3 | getstatic 参带体 | `p.Gs`：`ONE(Simple.A){…} TWO(Simple.B){…}`，ctor `(String,int,Lp/Simple;)` | 逐字段（[gs.base-mix.java](gs.base-mix.java)） | **保持逐字段**（[gs-mix.jarde.java](gs-mix.jarde.java)与前逐字节一致）。见下方边界登记 |
| 4 | 转发链断参（负例） | fam.jar 字节补丁：`Combo$1`/`Combo$2` ctor 转发位 `iload_3` → `iconst_2`（非本参常量） | 逐字段 | **整组逐字段拒绝**（[combo-broken-forward-mix.jarde.java](combo-broken-forward-mix.jarde.java)）：子类 ctor 转发证明（按参种类逐位 local-load）拒绝 → `constructor_bridge` Err → 组门拒绝 |

## 变体 3 边界登记（getstatic 参带体）

常量体义务参数化本身接受 Object 参（getstatic 限定名/null，实参种类判定复用 arbitrary-arguments 的 `prove_user_source_argument`），负例不在义务层拒绝：关系解析与常量步证明均通过。拒绝发生在**组证明的 structured initializer 义务**（`the initializer has no complete structured recovery`）：`<clinit>` 的恢复（jarde-java 初始化器构建层）尚不能把 `getstatic` 实参建模进构造步，`<clinit>` 恢复质量为 Fallback，初始化器 sidecar 无从逐写证明。该层不在本 change 声明的修改面（根 crate `src/enum_constants.rs`/`src/facade.rs`/`src/class_source.rs`）内，按边界如实登记，不放宽 structured initializer 见解（它同时保证"前缀之外无其他效果"）。后续若初始化器构建层补齐 getstatic 构造实参形状，本义务通道无需再改即可折叠该形。

## 变体 4 补充说明

补丁后的 jar 仍 `java -Xverify:all` 可运行且 main 输出不变（`7`/`12`/`0`——`p.Combo` 的带体常量不读回转发值），但折叠若发生将把实参拼写为与物理构造不符的源形态。拒绝义务在**子类/桥转发证明**（第 i 用户参必须是本参 local-load，种类按参数描述符），与 DT-12 既有"verifier 有效 ordinal 桥变化必须拒绝"的纪律一致。
