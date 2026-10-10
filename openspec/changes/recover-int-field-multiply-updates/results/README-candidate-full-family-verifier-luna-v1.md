# 双类族候选回放验收器 v1

`verify-candidate-full-family-root-luna-v1.py` 是针对 `candidate-root-luna-v2` 的独立验收器。它不调用候选采集器的 `main`，也不把采集器的成功布尔值当作证据；脚本只读取已经冻结的 baseline、候选产物、CLI 元数据和构建记录，最后写入 `candidate-full-family-root-acceptance-v1.json`，若该结果已存在则拒绝覆盖。

运行时必须由验收方传入冻结 CLI 与 metadata 的绝对路径及 SHA-256。脚本固定要求 metadata 绑定来源基线 `977f761d9f68c6cb4de02f42b060de5290a1a947`，产品仍标为未提交。它核对 10 个产品源、2 个测试源、由测试 `include_bytes!`/`include_str!` 推导的 canonical 文件集合及其当前字节哈希，并检查成功冻结 CLI 的九条真实构建命令和原始 stdout/stderr。

候选证据须闭合其文件清单，包含 8 次 `class-source` 回放、8 次完整源编译和运行、16 次新 class 的 `javap`。验收器重新解析保存的 `javap`，使用已接受的 EM23 v3 通用 helper 校对物理成员、字段/方法身份、访问标志及所有 source-map primary/derived BCI。默认与 `--evidence all` 的完整源码、每个方法正文和 source map 必须一致；嵌套类关系必须绑定两份原始 class owner，parent 投影范围必须精确切中嵌套类声明行。新编译 class 输出必须恰为 outer、`$A` 和 Runner 三个 class；完整 Java 源与原 Runner 在空 classpath/sourcepath 下编译，再以 `-Xverify:all` 执行，三路原始结果逐字匹配同 JDK 的原始基线。

验收器没有执行；本次只做了静态 schema 核对、内存语法编译和已接受 helper 对冻结 baseline `javap` 的只读解析预检。实际验收应由 root 在候选产物生成后运行。
