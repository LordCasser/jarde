# 嵌套枚举常量体折叠：负例与变体前后记录（任务 1.2）

全部 verifier 有效：各源以 `javac --release 8 -g:none` 编译，`java -Xverify:all` 运行通过（见 [threeway-fix.md](threeway-fix.md)）。冻结源在本目录 [fix/](fix/)，打包 jar 为 `variants-fix.jar`（`p.Weird$Name`、`p.TwoLevel$Middle$Deep`、`p.Holder3$Dual` 及全部兄弟类）。前后各为同一 jar 上基线 `ed41efe6` 二进制与修复后二进制的 `class-source` 输出（文本段 `1,/^}$/`），恢复文本 diff 为空 = 逐字不变。

## 1. 人为 `$` 命名的顶层用户枚举类（负例：不误伤）

`fix/Weird$Name.java`：`public enum Weird$Name` 为**源级顶层类**（类名含 `$`）。javac 视其为顶层：桥 marker 仍是自身首个常量子类 `p/Weird$Name$1`（`(Ljava/lang/String;ILp/Weird$Name$1;)V`）。

- 修复前：折叠（`FIRST { … }`、`SECOND { … }`）。
- 修复后：折叠，恢复文本 **diff 为空**。
- 判定：结构修复不含任何"名含 `$` ⇒ 嵌套"的推断，顶层语义保持。

## 2. 多级嵌套 `A$B$C` 枚举（登记：连带覆盖，非首片验收格）

`fix/TwoLevel.java`：`p.TwoLevel$Middle$Deep`（静态两级嵌套 + 抽象方法 + 常量体）。javac 的桥 marker 为最外层外围类合成匿名类 `p/TwoLevel$1`（空 `ACC_SYNTHETIC` 类）。

- 修复前：逐字段（`public static final p.TwoLevel$Middle$Deep X;`）。
- 修复后：**全量折叠**（`X { public int v() { return 1; } }`、`Y { public int v() { return 2; } }`），全部义务（桥委托、子类桥调用、独占普查、structured 体）按既有通道证明，三方 `java -Xverify:all` 对照一致（见 threeway-fix.md）。
- 判定：**超出首片"一层嵌套"验收格的连带覆盖**。修复本身不数层数、不做名段手术；`A$B$C` 满足本变更规格"枚举二进制名含 `$` 且义务满足 ⇒ 与顶层同语义"，义务不满足时仍逐字段（见第 3 例）。首片未将其列入验收矩阵，此处如实登记行为变化与验证证据；若 root 复核要求显式一层限制，需以结构事实表达（如 marker 外围类与枚举 `InnerClasses` 外围行核对），不应回到名段切分。

## 3. 嵌套 + 抽象/接口双形（义务不满足 ⇒ 保持逐字段）

`fix/Holder3.java`：`p.Holder3$Dual implements p.Holder3$IMath`，嵌套枚举同时声明抽象方法 `extra()` 且常量体实现接口方法 `compute` 与 `extra`（javac 需要两方法都落在体中）。

- 修复前：逐字段。
- 修复后：逐字段，恢复文本 **diff 为空**；证明状态 `Refused { reason: "a selected child lacks complete override bodies" }`——子类每个覆盖方法须绑定唯一基声明，"接口方法 + 自声明抽象方法"双基声明超出常量体切片的方法绑定词表，属**义务不满足的嵌套形状**，按规格保持逐字段呈现（物理字段 `public static final p.Holder3$Dual SQUARE;` 保留）。
- 判定：负例成立——修复未把义务不足的形状误翻为折叠。

## 4. 旧切片顶层 fixture（`Op`/`Mixed`/`Plain`）

`src/facade.rs` `enum_constant_body_relation_tests` 全部 26 项（含既有 21 项与新增 5 项）通过；`cargo test --workspace --tests --locked --no-fail-fast` 全绿（272 个测试二进制、2761 项，= 主线 2756 + 新增 5）。
