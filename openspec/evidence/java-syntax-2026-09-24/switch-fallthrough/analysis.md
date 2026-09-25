# 普通 `switch` 穿透：当前工作树独立验收

`present-proved-java-structure` 2c.4 的实现沿用现有正常流视图与 Region/AST：先按目标入口合并 case 标签；只在一个 case 的唯一正常后继链确实到达另一 case 入口时标记穿透，并在目标入口前停止前臂。穿透关系要求 BCI 向前、无环，最终输出的两个 case 必须紧邻（join-only 空臂也计入顺序）；随后按真实目标 BCI 排列标签，只对已证穿透臂省略 `break`。条件分支、多目标路径、非 case 共享块不能凭地址猜成穿透，继续使用现有拒绝边界。

root 以同一 Java 8 class 独立重放原源码、安装的 JADX 1.5.6、当前工作树 Jarde 的整类恢复、`javac --release 8 -g:none`、`java -Xverify:all`。Jarde CLI SHA-256 为 `44b83ade1c451e37ff1176dd4e2ad9d8e46e00729ba398d4ac608796b36de0aa`；本地 JADX 参考源码是 `2fb1b1638694`。旧冻结 CLI 的失败输出保留在 [原证据](../../java-syntax-2026-09-22/switch-fallthrough-order/analysis.md)，本次没有覆盖它的文件。

| 冻结输入 | class SHA-256 | 原/JADX/Jarde 逐行结果 | Jarde 引用 |
| --- | --- | --- | ---: |
| `SwitchFallthroughOrder`，`lookupswitch` | `511fa56a325aeb1c2c7781e3fb0c86b6640cc4c3436df7cbb5c05ce0f5943ad0` | 7/7 相同；`9:91:calls:1` | 0 |
| `TableswitchFallthrough`，共享 case 1/2 | `f45d78320286e1912b30f4b01524b17695a1cef269a0204b18d3d7191ce16f84` | 8/8 相同；`4:41:calls:1` | 0 |
| `StringSwitchVariants` 的第二级整数 switch | `234d6676b62896bb4a8865eb1bf21bed75bc9560132a2853e60ccc886c45f83d` | Jarde 对[原 18 行记录](../../java-syntax-2026-09-22/string-switch/variants/analysis.md)逐行相同 | 0 |
| `DefaultMiddle`，default 位于两个 case 中间 | `8a137f3884bf366a8a962c4bb2efb460b882aae2cd6198c459ff8f6de434add8` | 5/5 相同：`9:93`、`1:1`、`4:4`、`0:3`、`-1:3` | 0 |

四者的 Jarde 整类都能重编且通过 JVM 验证。第三项仍呈现 javac 的两级整数分派，不能据此宣称恢复了 Java 字符串 `switch`；那属于 `recover-string-switch`。定向 Rust 测试 `p3_switch_fallthrough`、相邻 `p3_char_switch`、`p3_switch_value`、`p3_exception_scope`、`p3_loop_test_values` 共 10 项通过；新增 fixture 的三份 class 哈希与上表一致。`DefaultMiddle` 的两次穿透均无 `break`，`case 1` 后的独立 `break` 仍在。JADX 的相邻关系排序失败时会清掉穿透关系并告警，Jarde 采用可证明的入口链与最终邻接条件，不以可编译文本代替等价性。
