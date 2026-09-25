# root 独立整类重放：null 资源头

`run_audit.py` 以独立复制的 Java 8 源重新编译原 class，把冻结 CLI `/tmp/jarde-cli-null-root-final` 和 JADX 1.5.6 的完整输出原样交给 `javac --release 8`，仅在编译成功时用 `java -Xverify:all` 执行 runner。CLI SHA-256 `30481c1449b5a1869773d21b934789d42bb1d6a9cab8d81f873d50055af4e34c` 在运行前后相同。每组的 `audit.json` 保留状态、class 哈希、逐行结果与跳过原因；总表为本目录的 `audit.json`。

| 输入 | 原 class | JADX | jarde | 原/jarde 相等 |
| --- | --- | --- | --- | --- |
| core（nullable factory） | 编译/运行 0，3 行 | 编译/运行 0，3 行相等 | 编译/运行 0 | 是 |
| multi-resource | 编译/运行 0，1 行 | 编译/运行 0，1 行相等 | 编译/运行 0 | 是 |
| null-resource，670 B | 编译/运行 0，1 行 | 编译/运行 0，1 行相等 | 编译/运行 0 | 是 |
| exceptional，680 B | 编译/运行 0，1 行 | `javac` 1，未运行 | 编译/运行 0 | 是；primary 同一对象、suppressed 0、close 0 |
| ordinary-only，441 B | 编译/运行 0，1 行 | 编译/运行 0，1 行相等 | 编译/运行 0 | 是；保留真实 RuntimeException catch |
| same-type 混合边界 | 编译/运行 0，1 行 | 编译/运行 0，1 行相等 | `javac` 1，未运行 | 未比较；手写无抑制清理的普通 null 局部变成 `Object.close()` |

JADX 的 exceptional 方法发射 `throw th` 却未声明 `Exception`；日志在 `exceptional/jadx-javac.stderr`。同类型混合类的失败只在普通 null 局部方法，不属于资源头恢复。`negatives/replay.py` 使用同一 CLI 独立核对破坏抑制链、仅继承接口与普通用户 catch 的方法级分类；三个 class 的 SHA/指令及结果见 `negatives/summary.json`，未因引用输出而假称可运行。永久 class、Code 哈希和 BCI 见 `tests/fixtures/p3-null-resource/README.md`。
