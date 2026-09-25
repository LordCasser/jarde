## 1. 冻结正反证据

- [x] 1.1 保存 Java 8 `new int[]{1,a++,a*2}` 的源码、class、javap、五组原/JADX/Jarde 完整类与重编结果；[分析](../../evidence/java-syntax-2026-09-25/array-postfix-element/analysis.md)确认 JADX 等价但不输出 `++`、旧 Jarde 缺 return。
- [x] 1.2 将唯一 `iinc slot,+1` 改成 `+2` 形成 verifier-valid class，验证输入 0 原 JVM 输出 `[1,0,4]`、当前 Jarde 保守拒绝，并保存 class SHA/运行证据。
- [x] 1.3 建立永久 fixture 与不同 slot/额外 use 至少一项 verifier-valid 控制，验证原 class 可执行、恢复不能误认 `local++`；登记 fixture 与语料指纹。本 fixture 的八个应登记文件均在清单中，路径与长度由 root 独立核对；其它 121 项清单漂移另列债务。

## 2. 局部后置自增元素

- [x] 2.1 在现有数组链证明中只接纳同块相邻 `iload` 旧值、同槽 `iinc +1` 和唯一当前元素 store；定向测试正例结构恢复及 1.2/1.3 控制拒绝。
- [x] 2.2 复用现有 `PostIncrement` AST 发射局部 `++`，把 load/iinc 与元素位置一起提交并抑制独立 iinc 语句；Java 8 整类编译正例、原/Jarde 五组验证运行逐行一致，后一元素读更新值。
- [x] 2.3 验证所有真实 BCI 来源、一次求值、预算/取消的原子回退，并复跑既有一维/多维数组初始化、后置字段/数组值及普通局部 `iinc` 回归。

## 3. Root 独立验收

- [x] 3.1 root 独立构建 CLI、读取冻结与控制 class，完整类 Java 8 重编及 `java -Xverify:all` 对照，确认正例五行与原/JADX 相同、`+2` 控制不被误写 `++`，记录 CLI/class SHA；见 [独立验收](verification-root.md)。
- [x] 3.2 root 审读 SSA 旧/新局部、求值顺序、来源与拒绝边界，运行定向回归、`cargo fmt --all -- --check`、适用 Clippy、`git diff --check`、OpenSpec strict，记录未完成项并清理临时 Cargo target；见 [独立验收](verification-root.md)。
