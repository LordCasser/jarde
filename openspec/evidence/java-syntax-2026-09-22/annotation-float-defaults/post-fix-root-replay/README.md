# F/D 注解默认值：主代理独立验收

root 从当前源码重建并冻结 `/tmp/jarde-cli-ann-fd-root-final`，SHA-256 为 `f2fb24241c2692dff53060576e185eb282617a6881e81900140bb2eb62006544`。先把父目录的原始夹具复制到 `/tmp/jarde-ann-fd-post-root-jjzieyzl/case`，把其中审计脚本的 CLI 哈希和修后预期结果改在**副本**里重跑；本目录 `run_audit.py` 是这一修后脚本，未修改父目录修前证据。原 `FloatDefaults.class` 仍为 293B、SHA-256 `c257ba990cb21bd1769ad7a949e65a8338934a3b7e202e0bc9ab964ab509ecad`。完整原/JADX/Jarde 注解类与 runner 均通过 `javac --release 8` 和 `java -Xverify:all`，四行 raw bits 逐项同为 `80000000 / 1 / 7f800000 / 7ff8000000000000`。Jarde 文本和 JSON `text` 字段相同；`summary.json`、类文件、`javap`、源码和日志均保留在此。

`run_patches.py` 使用父目录两份已冻结的**单常量池项补丁**，不把它们当普通 Java 源码。原 patched class 在全校验下分别保留 `ffc00000` 和 `7ff8000000000001`。Jarde 对各自异常 NaN 的方法省略默认值，其余三项仍有默认值；两套完整类与未改 runner 可以编译，runner 因那个缺失默认值而 NPE。`patches/summary.json` 和完整生成源码/JSON/日志记录两组实际结果。

`boundaries/` 是 root 从代理交付的独立 Java 源码重新编译、再用冻结 CLI 原样生成的执行对照。`FloatingBoundary` 八个负零、次正规、最大有限、无穷及 canonical NaN 的 raw bits 与修后 Jarde 逐行相等；`FloatingArrays` 的 F/D 数组四项也逐行相等。数组单池项 payload NaN 补丁通过全校验并读出 `7fc00001`；Jarde 只拒绝整个 `floats` 默认值，保留 `doubles`，生成类可编译、runner 对缺失默认值 NPE。`boundaries/summary.json` 给出 class SHA 与输出。第一次边界脚本组合时将补丁输入放在 `original/` 而补丁器要求 `arrays-original/`，因此只发生了路径错误；将同一 class 字节复制到预期目录后补丁与后续对照全部通过，`patch.log` 已记录最终成功重放。

重放基线和补丁时需保留父目录的受控补丁文件及上述冻结 CLI；运行 `python3 run_audit.py`、`python3 run_patches.py`。产品代码没有执行被分析的目标程序；`javac`/`java` 只用于这组自写验收夹具。
