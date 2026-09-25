# 2.3 的 `FinallyLeadSnapshot` 子切片

冻结类 `FinallyLeadSnapshot.class` 是 287 B，SHA-256 为 `187546178ad20c70ac00b8ab9df464a6cb9bf617dae6794dc16884463a003a3c`。异常表只保护 `[5,9)`；BCI `[0,5)` 是一条完整的静态字段赋值。当前 claim 只在这段 lead 恰为完整字段赋值、处于同一块、没有任何异常表覆盖且保护区边界栈为空时，使用已有 `Plan.lead` 将其放在 `try` 前。`Plan.body` 仍为 `[5,9)`；保存返回值及两个 cleanup 副本的原证书保持不变。`explained` 与 owned 检查包含 lead，保证被认领的物理指令没有遗漏或重复。

聚焦测试 `p3_finally_straight` 5/5、`p3_guard` 13/13、`jarde-java --lib finally_copy_tests` 8/8 通过。新增测试核对前置赋值、返回快照、唯一 cleanup、来源 BCI，并让精确异常表项 `00 05 00 09 00 10 00 00` 的 `start_pc` 从 5 变为 2：变体 SHA-256 `6a5f671b32bc86d7a8c990d7a0af26025bb8502c42ebae163e763d1ec50dbf63`，JVM 验证有效且仍输出 `41:99`，但跨边界栈值使 Jarde 保持引用。已有冻结类的范围缩窄、副本分歧和分支正文也保持引用；重新编译的覆盖型 `FinallyCompletion` 907 B 类 SHA-256 为 `2b8902d998065d2d1747abaeba386e006cdf5ff7f1311abb28506e8d8dc0cc94`，`finallyReturns` 与 `finallyThrows` 仍引用。

独立 Cargo 目标构建的 CLI 已复制到 `/tmp/jarde-finally-lead-cli`，SHA-256 `780192989e12627bc4c4c6a0fea79d7e43c9bdefd1bc6c679d647b5920d37e9f`。使用该二进制对冻结正例运行 `class-source --evidence all`，生成的完整 Java 类直接通过 `javac --release 8`；runner 的 `java -Xverify:all` 输出 `41:99`，与原类一致。`openspec validate recover-proved-finally-cleanup --strict` 通过。

本记录只验收 lead 子切片。`ImplicitCleanup` 的分支正文和 2.3 其余完成路径尚未在此闭合，任务 2.3 保持未勾选。
