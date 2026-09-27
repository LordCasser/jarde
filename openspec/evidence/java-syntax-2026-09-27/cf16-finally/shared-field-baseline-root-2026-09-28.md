# CF-16 静态字段增量清理：主线冻结基线

主线 `d64f2c68` 上，root 从 `tests/fixtures/p3-shared-catchall-finally/SharedFinally.java` 和 Runner 重新执行 `javac --release 8 -g:none -Xlint:-options`、`java -Xverify:all`，得到 `normal:1 / caught:1`。重编 class 与归档逐字节一致，SHA-256 `8677378307e4d1bdd422b6dbe6860c9fe1aca6c3eff944258c19078b71bfd0bd`。归档 `javap.txt` 的 ordinal 0/1/2 依次是 `[4,21)→31 IllegalArgumentException`、`[4,21)→45 any`、`[31,35)→45 any`；三份字段读-加一-写回位于 `21,24,25,26`、`35,38,39,40` 和 `46,49,50,51`。

固定 JADX checkout `2fb1b16386941660fda07e9017285aec40fcb37f` 的归档完整类源码重编及验证运行得到 `normal:2 / caught:1`，因为正常 try body 和 finally 各更新一次字段。root 用当前主线重新构建 CLI，从归档 class 实时生成 Jarde 完整源码，与 `tests/fixtures/p3-shared-catchall-finally/jarde/SharedFinally.java` 逐字节相同（SHA-256 `043863832ebf76812fd3a8162df8a09f1cb29cc3a1c817691d0c9d72e1988cbb`）；`handled` 保留 `@bytecode 0 8 18 31 45` 和 slot 1 跨引用诊断，完整源码在第 21 行因缺少返回而不能重编。Jarde 此时是安全拒绝，不是语义等价。

源码 SHA-256：原类 `aaf5d739ce0fa92bebc1d64bd1bc7141ed19ba710cee541e118c54b9e63fe0a2`、Runner `84b706b96decd7b86f5e676d36825f45b29eb2d917585a5d68919f5718a54417`、固定 JADX 类 `ba09b9fb694c787ead3f35e36936c41898268ce99ccca5db0ce528663020702d`。固定 JADX `MarkFinallyVisitor.java` SHA-256 `9999700ad951b8a309c786bf8efbb3d5d169b1696b25a2682a584b6d6e6db63b`：其副本匹配可供候选定位参考，但本样本的正常路径计数错误，不能作为效果验收 oracle。
