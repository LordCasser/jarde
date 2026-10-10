# CF-07 frozen baseline 独立核验入口

`verify-baseline-root-v2.py` 是只读验收器；执行时只读 `baseline-root-v2` 与已固定的工具输入，不调用 Git、JDK、JADX 或 Jarde CLI。root 应审阅后实际执行，结果写到同级 `independent-verification-root-v1.json`，不会进入 `baseline-root-v2` inventory。

脚本固定输入源码、Runner、collector v2（SHA-256 `87c693a53f31898374d5c963c6fbe14b550db20439e0409486b9d191a00cec8f`）、CLI/metadata、JDK manifest 和 JADX pins。它逐文件核 118 项 inventory 与闭合、29 条命令的退出记录及 stdout/stderr raw 哈希，再以真实 raw 对照 2 个原始 oracle 与 4 个 JADX、4 个 Jarde case 的同 JDK exit/stdout/stderr；同时核各源文件、共同 Runner、空 classpath/sourcepath 和每次新编译恰为两个 class 文件。

物理方法和指令 BCIs 从两份已捕获 `javap.txt` 用局部正则解析，不从 Jarde source map 反推。脚本核四个方法的精确 flags、所有 origin 的物理方法 owner/BCI 与 UTF-8 source span、default/all 整类及逐方法文本和 source map 一致。已知来源缺口按原样记录：`andWhile` 的 `goto@15 → 2`；另有 `counted` 的 BCIs 20/30、`lastIndexOf` 的 BCI 25。它们只是冻结旧 CLI 的基线观察，脚本不会据此宣称全指令来源通过或任何新候选已接受。

当前只做了静态审阅准备，未执行该验收器，也未运行 Git、JDK、JADX、Jarde CLI 或 Cargo。
