# DT-29 P1 接收者绑定

运行 `JARDE_CLI=/path/to/jarde-cli python3 replay.py`。脚本只在临时目录编译和变异 class，校验固定 JADX revision 与组合输入 SHA-256，并将稳定结果写入 `results.json`。

独立夹具包含 A/B/C/D/根类五个物理类。B 隐藏 A 的四个同名字段，根类同时声明私有 `bits(A)` 和 `bits(B)`。原始、固定 JADX 与 Jarde 全源码分别执行 `javac --release 8 -g:none` 和 `java -Xverify:all`，结果均为 `101:true:false:true:false`。这同时证明 A 字段保持原身份，B 隐藏字段不被写入，根类三次调用仍绑定 `bits(A)`。

逐点证据位于 `results.json`：独立 C/D 的字段与 accessor 是 BCI `2/7/12/17`，根类三次私有调用是 `14/31/57`；固定组合输入 C/D 同为 `2/7/12/17`，根类调用是 `14/31/74`。脚本检查 Jarde source map、文本和 `javap` 的物理 CP 目标。固定组合输入的 D 泛型声明与 `bits` 跨块拼接属于 P2/P3，本包只检查 P1 对应方法和调用点。

拒绝用变异 class 覆盖错父类、字段 CP owner/name/descriptor、方法 CP owner/descriptor、错 SSA 接收者、跨包 protected、private/static/final 字段、重复 accessor；每项均保留原始 BCI 的 `@bytecode`。`method_bodies=1` 明确停止。Rust 定向测试还验证私有目标存在 `bits(B)` 重载时唯一选择，以及声明歧义、预算与预先取消时不颁发证书。
