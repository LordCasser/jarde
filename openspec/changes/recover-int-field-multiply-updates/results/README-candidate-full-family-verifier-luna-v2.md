# 双类族候选回放验收器 v2

`verify-candidate-full-family-root-luna-v2.py` 只读取 `candidate-root-luna-v2`、两份已接受 original baseline、冻结 CLI/metadata 与 validation-build v2 记录；它不调用候选采集器入口，也不把采集器的成功字段作为唯一依据。命令行必须显式提供 CLI 与 metadata 的绝对路径及 SHA-256。metadata 必须绑定来源提交 `977f761d9f68c6cb4de02f42b060de5290a1a947`，并标记产品尚未提交。验收结果写到 `candidate-full-family-root-acceptance-v2.json`，拒绝覆盖已有结果。

验收会重新校验 10 个产品源、2 个测试源和测试 include 闭包中的 canonical 文件哈希，核对 validation-build-root-v4 的 9 条冻结构建命令、原始 stdout/stderr、记录的 HEAD、资源限制和环境覆盖，并从原始输出重解析测试汇总。候选证据须以完整文件清单闭合，具有 8 次 class-source 渲染、8 次完整源 fresh compile/run 和 16 次 fresh javap。默认与 `--evidence all` 的完整源码、每个物理方法正文及 source map 必须逐项相同。

每个物理方法都由已接受的 EM23 v3 helper 对照原始 javap 校验成员身份、flags、source-map primary/derived 来源 BCI 与源码 span。nested owner relation 必须同时对应原 outer/`$A` class；父源码投影的 span 必须恰好切中 `private static class A extends java.lang.Object {`，且两个 class-definition anchor 与真实 owner 一致。fresh class 输出需实际枚举为 outer、`$A`、Runner 三个 class，outer 与 child 的 fresh javap 成员/flags/descriptors须匹配原物理类。完整产品源码与原 Runner 在空 classpath/sourcepath 下编译，之后由 `-Xverify:all` 执行，并逐字匹配相同 JDK 的 original 三路输出。

字段读取/写入也按物理 owner、描述符和 BCI 精确检查：`test1` 保留 `a@1/a@5` 两次 receiver 读取、`f@8` 读取和 `f@13` 写入；`test2` 核对 `a@1`、`f@5` 读取及 `f@10` 写入。multiply controls 的 `multiplyDivide` 进一步核对全部五个 field access、结构化 `*= 8 / arg1` 和更新后的字段返回读取。

v2 保留 v1，并修正 root 静态审查发现的问题：原始物理 census 从已冻结 javap 文本解析并传入后续比较；fresh javap 路径包含 `em23/` 包目录；`test1` 按真实字节码把两次读取归属到外层 `a` 字段，而不是误计为两次 `f` 读取。另从 build raw stream 重解析测试汇总并核对 guard/env/HEAD 记录，并绑定 validation-runner-v4 的路径/SHA。v4 目前仍待 root 运行，因此验收器不声称构建通过。只读预检已确认两份 baseline 四个 JDK legs 的物理成员、flags、BCI，以及 nested declaration span 和 owner anchors；没有执行最终 verifier、collector、CLI、JDK 或 Cargo。
