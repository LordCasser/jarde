# Unicode 标识符巡查（2026-10-05 root）——呈现缺陷级：CJK 标识符被替换为 `__`

## 发现：`is_java_identifier` ASCII-only——中文字段/方法名渲染为 `__`（同名碰撞不可编译）

- **方法体内的中文引用正确**（`UT.描述`、`方法(21)`、`local1.名字`、池名 `UT$内部类` 全部如实）；
- **声明层被替换**：字段 `变量`/`描述` 声明渲染为 `static int __;`（两个 `__` 同名碰撞）+ 方法 `方法` 签名 `static int __(int)`；注释自曝 raw name：`the field's raw name \`/u53D8/u91CF\` is not a Java identifier`；
- **根因单一**：[names.rs:125](../../../../crates/jarde-java/src/names.rs) `is_java_identifier` 用 `is_ascii_alphabetic`/`is_ascii_alphanumeric`——JLS Character.isJavaIdentifierStart/Part（含 CJK）未实现；
- **手工还原证明**：把 3 处 `__` 替换回中文名后全链编译 exit 0、行为 `变量=1/42/中文` 逐行一致（[verify-manualfix](results/verify-manualfix-UT.java)）——其余渲染层全部健康；
- jadx 以 `f0/f1/m0` 驼峰映射规避同题（自身也不写 CJK，但无碰撞）。

## 影响面

字段/方法/类声明层的 CJK/非 ASCII 标识符（真实安卓代码混淆保留中文的场景存在）；方法体引用不受影响。

## 处置

**呈现缺陷窄片立项**：`is_java_identifier` 按 JLS Character.isJavaIdentifierStart/Part 判定（ASCII 超集扩展，raw name 已是合法池 UTF-8）；无新机制。
