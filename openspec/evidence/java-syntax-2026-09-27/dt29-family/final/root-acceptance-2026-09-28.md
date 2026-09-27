# DT-29 `TestFieldCast` 完整类族：主线独立验收

在生产提交 `16f42579` 的主线，以 CLI SHA-256 `a9f10d5db5de84925174d248addff35bebbf1d8525c02a6f5da5962660905cd7` 独立运行[固定九类回放](../baseline/replay.py)的 `--require-jarde` 门槛。[结果](main-16f42579/summary.json)保留固定 JADX revision、源码和九个物理 class 的 SHA-256、三方编译/运行/反射结果及每个方法的物理来源；[九份 Jarde 源码](main-16f42579/jarde-source/)可直接审阅。原始、固定 JADX、Jarde 的完整类源码均以 `javac --release 8 -g:none` 重编，`java -Xverify:all` 输出都为 `runnable:1111:0000:1111:ClassCastException`。单独反射 Runner 三方均输出 `1:T:dt29.FieldCast$B:T:dt29.FieldCast$B`，约束 D.set 的类型变量、上界、泛型参数及擦除。九份 Jarde 源码均无 `@bytecode`；C/D.set、根类 run/bits 均为结构化正文，无缺失 return。

三个独立包的主线复跑也通过：[P1 接收者绑定](../p1-receiver-binding/README.md)的五类 A/B/C/D/根类三方输出 `101:true:false:true:false`，隐藏字段和竞争 `bits(B)` 仍选择准确 A owner/私有 `bits(A)`；13 项错父类/CP/SSA/访问性/accessor 负例保持引用。[P2 有正文泛型 void](../p2-generic-void/replay.sh)的三方 Java 8 重编、验证运行、反射和四项负例通过，`method_bodies=1` 为 partial 停止；固定 D 的 `$B` 上界只复用其物理参数拼写，并以当前 D 的唯一 `InnerClasses` 表项证明同一 outer 与合法直系成员名，通用 `$` 拼写门未放宽。[P3 四段条件拼接](../p3-branched-concat/README.md)的四个位型和替代字段名/`Y/N` 字面量三方一致，带可观察效果的 getter 输出 `YYYY:1:false:false:false`，七项别名/φ/效果/异常/重载/缺失负例拒绝。

最终完整类族的 source map 包含 C/D.set 的 BCI `2/7/12/17`、run 的 `14/31/74` 和 bits 的 `8/21/25/38/42/55/59/72/78`；P3 的全部 33 条物理指令亦在来源中。主线 `cargo test -p jarde --lib --locked` 为 **151/151**，`cargo test -p jarde-java --tests --locked` 全部通过（库测试 **233/233**，其余测试二进制均 0 失败）；`cargo fmt --all -- --check`、`cargo check --workspace --locked`、`git diff --check` 和 `openspec validate recover-dt29-fieldcast-family --strict` 通过。专用 Cargo target 已用 `cargo clean` 清理 17,508 个文件、6.7 GiB 构建残留。

这关闭固定 `TestFieldCast` 多成员组合的已证差距，不把 DT-29 的所有引用强转变体推定为完全覆盖。P1 限选中直接父类与实际 SSA 接收者；P2 限完整直线 void 正文、准确擦除及源名证明；P3 限四条件/33 指令的完整 builder 方法。扩大继承深度、泛型正文或拼接形态需另立证据与任务。
