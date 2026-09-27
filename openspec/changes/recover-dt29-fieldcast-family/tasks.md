## 1. 固定基线和分包边界

- [ ] 1.1 用当前主线重新回放固定 `combined/inputs/FieldCast.java`、Runner 与三个接口的全部九个物理类，记录原/JADX/Jarde 的 Java 8 重编、`-Xverify:all`、每个失败方法及 BCI；校验固定 JADX revision/源码 hash，并产出可重复脚本和基线报告。

## 2. P1：跨接收者字段和私有调用

- [ ] 2.1 在现有选中类层级及字段/调用证书上逐 BCI 证明 C/D 的 B 接收者访问 A 四字段和 A.access$002、根类三次 B 实参调用私有 bits(A)；独立 A/B/C/D/根类完整 fixture 原/JADX/Jarde Java 8 重编、验证运行且相关方法无 `@bytecode`。
- [ ] 2.2 用隐藏字段、错父类/CP owner/name/descriptor、跨包访问、private/static/final 变体、错 SSA 接收者、accessor 歧义与竞争 bits(B) 重载验证安全拒绝、来源和预算/取消；相关 Rust 定向测试通过。

## 3. P2：有正文的泛型 void 声明

- [ ] 3.1 复用 Signature 解析与擦除核验，让 `<T extends B> void set(T,boolean)` 的获证参数使用保留源级泛型头，物理正文仍以 B 解释；独立完整类族原/JADX/Jarde Java 8 重编、验证运行及反射四项元数据一致。
- [ ] 3.2 用错上界/擦除、未绑定 T、不兼容正文使用、同类重载和预算/取消验证拒绝与来源；泛型声明及相邻 class_source Rust 测试通过。

## 4. P3：跨块四条件拼接

- [ ] 4.1 证明 bits 的唯一 builder、四个条件值与 φ、四次 append(String)、四个字段读取和最终返回的块/SSA/效果顺序，复用现有拼接或最小语句投影；独立完整类族原/JADX/Jarde Java 8 重编、验证运行 `1111/0000/混合位型` 一致且无 `@bytecode`。
- [ ] 4.2 用 builder 别名、φ 复用/交换、额外效果或异常边、不同 append 重载、缺 getter 和预算/取消验证安全拒绝及物理来源；拼接及相邻方法 Rust 测试通过。

## 5. 完整类族集成验收

- [ ] 5.1 合入三个工作包后重新生成固定九个物理类的完整 Jarde 源码；原/JADX/Jarde 均以 `javac --release 8 -g:none` 重编，`java -Xverify:all` 精确输出 `runnable:1111:0000:1111:ClassCastException`，所有相关方法无 `@bytecode`/缺失 return，D 反射泛型签名和逐 BCI 来源正确。
- [ ] 5.2 重放三包独立负例与已通过的 DT-29 父字段/接口 cast 控制、相关 Rust 测试、`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`git diff --check` 和 `openspec validate recover-dt29-fieldcast-family --strict`；root 独立记录结果、清理专用 Cargo target 并提交推送。
