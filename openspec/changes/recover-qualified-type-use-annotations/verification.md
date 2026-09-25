# Verification

## 本轮实现

- `jarde-reader` 为两种 Runtime*TypeAnnotations 解析完整 `target_type`、原始 `target_info`、`type_path` 和共享 annotation 值树；每项保留属性内物理顺序，要求精确到达属性末端。localvar target 逐行 poll 并计入 `ResultItems`，不按不可信表长预分配。
- 字段/方法成员读取按自身属性表物理顺序交接声明、形参和类型注解 shell；类级/Code 属性不转移到成员。JSON 在 shell 旁保留完整目标/路径/值及拼写或拒绝。
- 仅空路径、非数组、具有限定内部位置的引用类型可以写出。形参逐 descriptor 位置注入；声明冲突按字段、返回、各形参分别判定。其余位置保留 shell 与事实并拒绝。

## 定向测试

- `CARGO_INCREMENTAL=0 cargo test -p jarde-reader --test type_annotation_facts`：4 项通过，覆盖 visible/invisible member target、空 shell、预算、取消、完整 localvar target_info、localvar 项预算以及截断表拒绝。
- `CARGO_INCREMENTAL=0 cargo test -p jarde --test class_source`：42 项通过，覆盖三种可拼写位置、宽槽前参数的 descriptor 索引、RuntimeInvisible shell、字段/方法/参数归属冲突、重复目标原子拒绝、错置 owner、越界形参、primitive、单段名、数组及非空路径拒绝，并回归已有声明注解和方法正文。
- `python3 -m py_compile openspec/evidence/java-syntax-2026-09-22/type-use-annotations/run_audit.py` 通过。

## 冻结 CLI 重放

使用冻结副本 `/tmp/jarde-cli-type-use-recovered-d5309996cbcf1da4`，SHA-256 `d5309996cbcf1da444550e19bc237e89e6aa13a59020f1a075b12b08643259f6`。重放命令通过 `TYPE_USE_AUDIT_OUT=openspec/evidence/java-syntax-2026-09-22/type-use-annotations/generated/recovered` 将结果放在新目录，不覆盖修前记录。结果见该目录的 `summary.json` 及逐命令 `.stdout`、`.stderr`、`.exit`。

原类编译和 `-Xverify:all` 成功，反射为 `field / return / parameter / [o, k]`；JADX 仍编译、验证成功，但三种类型注解均为 `null`。修后 Jarde 的完整源码集仍因既有 runner 缺少返回值而未通过 javac，因此没有整类运行结论。未编辑的修后 `TypeUseSubject.java` 与原 `TypeMark.java`、runner 独立以 Java 8 编译并通过 `-Xverify:all`，反射恢复为 `field / return / parameter / [o, k]`。该隔离 class 的 `javap -v -p` 只显示三个原有 `RuntimeVisibleTypeAnnotations` 目标和值，未出现 `RuntimeVisibleAnnotations` 声明属性。

另以 `RetentionPolicy.CLASS` fixture 对照原类与 Jarde 源码重编译结果。`javap -v -p` 两侧均保留 `RuntimeInvisibleTypeAnnotations` 的 `FIELD`、`METHOD_RETURN`、`METHOD_FORMAL_PARAMETER(param_index=0)` 目标和值 `field / return / parameter`；两次反射均为 `0,0,0`。双目标 placement fixture 的 Jarde 输出以 Java 8 成功重编译，重编译前后的 `javap -v -p` 均保留字段和方法声明属性与各自类型 target/value，包括普通前缀同时产生声明和类型属性的边界，以及限定名内部仅产生类型属性的边界。

## root 独立验收

root 审查了有限 target_info 格式、成员 owner/descriptor 形参位置、同位置声明冲突、空路径限定名内部的 type-only 写法，以及属性长度、localvar 表逐项预算和取消。补充修正形参按文本替换的歧义、跨平面冲突归属及 localvar 表收费；严格 Clippy 后复用已有 `MemberAttributes`，没有增加重复结构。

最终独立重建 CLI 冻结于 `/tmp/jarde-cli-root-final-accepted`，SHA-256 `d7520a08a37b54ef579d42c770f5b512af17ddb11b05050568458759c78dbf16`；以 `/tmp/jarde-type-use-root-postrefactor` 为新输出目录原样重放。原类和 JADX 完整类都能编译执行，分别为 `field/return/parameter` 与三项 `null`；Jarde 完整类仍因无关 runner 缺返回而 javac 失败，隔离 subject 在 Java 8 下编译、`-Xverify:all` 及反射均恢复三项原值。不可见属性与 placement 对照也保持修后结果。最终类源码 42/42、reader 160/160 及类型属性 4/4、CLI 16/16、corpus fingerprint 5/5 和 reader census `(151, 1086, 98, 346, 8)` 通过；`cargo fmt --all -- --check`、严格 reader Clippy、root lib Clippy（仅保留无关 `region.rs` 既存 `type_complexity` 例外）、`git diff --check` 与此 change 的 OpenSpec strict 通过。双目标声明前缀可能新增类型属性的债务独立保留于 design，不混入本轮。
