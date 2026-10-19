# 定向验证与完整源码对照

root 已接受并逐成员复开 capture.tar.gz：357 个 regular payload 加内嵌 inventory，SHA `8200e38eb0646cae1cdc2c1017c6b74c1a0d65c1daf02fcc910c86e9f7a02b17`。所有文件字节、mode、独立 inventory 与原证据一致；唯一不打包的是已冻结 CLI，63983392B/SHA `9ea9177b8992c8444c9f9af0158ca7bc99b3e7dfee4696b342aea7da2e295167`，其 pin 保留。

最终采集72命令：debug/no-debug × class/plain JAR × default/all 共8 API；16次完整源码 JDK8/23 编译成功；32个正向 VM × 六有限用例共192次观测全部与原件一致；16个错 CodeSource 根的负向 VM 在 CASE/目标调用之前拒绝。完整 outer/child/check 不裁剪，check/JUnit 不调用。标签跳出含 hit-before-null-row 控制正确，布尔值打印差异消失。三方法保留、目标 structured，无 fallback；map 由34/42变41/42，BCI44仍缺，不把 Evidence Complete 当全指令覆盖。

原件及 JADX 精确复用已接受的[完整基线](../../../../evidence/java-syntax-2026-10-19/cf09-testbreakwithlabel-original-root-v1/README.md)，不冒称重跑。原始/JADX JDK8 框架版本失败完整保留在该基线归档；本归档保留当前采集引用、全部 prep/真实失败、修正和接受器原始流。归档脚本 inventory 对“controls 包含原始/JADX基线”的泛称不表示重复打包完整基线，权威入口是上述持久目录。

定向验证分失败保留续跑：Java lib509/0；hoisted9/0、Boolean contexts13/0、low-bit边界1/0与契约5/0、mixed local5/0；新公共3/0；bitwise5/0/1旧ignored、shortcircuit5/0/3旧ignored；Reader1/0、fingerprint显式重建1/0及正常5/0/1旧ignored，fmt/CI同款Clippy/最终CLI构建通过。v1旧hoisted期望失败、v2包名含boolean导致负向断言误报、v3三target通过后清理器只接受单executable的错误均保留；生产代码未为修测试/清理器改变，只续未执行步骤。全部 finally exact clean、target absent、源码冻结，峰值均低于1GiB。

真实准入 unit 验证全访问闭合、重复/遗漏写入拒绝、path缺失拒绝、实际 proof budget cut 与预取消；公共取消只称预取消。候选v1全局Phi传播被root拒绝，v2仅按需检查实际读取的一层Phi；不新增solver/CFG/Region或放宽旧concat gate。独立reviewer v2的receipt变量遮蔽绑定错误被root拒绝；v3仅修绑定后重审同一采集，没有重跑目标。

完整 fresh workspace、最终产品自身CI及产品Git绑定尚待，不能用本定向结果关闭任务3.2。
