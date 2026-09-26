# 2.3b Root 验收：清理块所有权与资源体局部作用域

实现提交 `7124e2f8`。TWR Guard 只扩展认领完整落在已证 cleanup 指令集合内的非连续 handler 块，并核对其入出边属于已证异常表行或清理链；`slot_uses` 仅按这些指令 BCI 排除 synthetic 读写。体内存值、声明、读取及返回仍进入正常语句规划，不因同一物理 slot 在 handler 中存过 Throwable 而混淆。向 primary handler 增加一条未证明的竞争 catch-all 行时，恢复保持拒绝。

真实单资源 `runSaved` 现写出 `try (java.io.ByteArrayInputStream local1 = open()) { arg0 = local1.read(); int local2 = arg0; return local2; }`，`run` 同样把局部声明与返回留在资源体内，二者无 fallback。代理用 `javac --release 8` 重编两种恢复方法，并在可控 close 正常/抛 `IOException` 的等价 Java 结构上比较输出，分别为 `42` 和 `java.io.IOException:close`；冻结 class 自身的 `open()` 固定返回普通 `ByteArrayInputStream`，抛错 close 路径不是对冻结二进制的直接注入。

Root 在代理提交的隔离工作区用独立 Cargo target 复跑 `p3_twr_return_tail` 5/5、`p3_multi_resource_twr_geometry` 4/4、`p3_guard` 13/13，共 22 项；`cargo fmt --all -- --check` 与 `git diff --check` 通过。代理的 strict Clippy 被既存 `enumswitch.rs`、`region.rs`、`report.rs`、`build.rs`、`reuse.rs` 旧 lint 阻断，本次新增代码未产生所列警告，整仓静态检查仍留在 3.2。

双资源冻结样例仍因资源头 BCI 0/10 的 `new; dup` 生产者未归属而拒绝，2.4 及完整四路径对照 3.1 保持未完成。此次验收不扩大为全异常区局部作用域恢复。
