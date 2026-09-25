# Reader/query 签名语法与擦除接缝验收

reader 的共享签名 parser 按需完整消费 JVMS 属性、保留方法结构并以字节数、节点数、深度和既有预算限制输入；query 从同一个解析结果依原顺序列出类引用，不再维护第二份 grammar，也不作为 decompiler 的依赖。方法擦除证明逐位置核对物理 descriptor，解析方法局部类型变量的第一界并拒绝未知/类级变量。`Signature` 没有 `^` 后缀时保留同成员 `Exceptions`；只有显式后缀才核对异常擦除和顺序。

代理测试通过：reader 签名单元 7/7、冻结类属性集成 3/3、query 泛型 grammar 及冻结类回归。root 独立执行 `cargo test -p jarde-reader --test method_signature_proof --locked`，3/3 通过；五个 JVM 可验证负例的固定 SHA 与 [1.2 证据](../../evidence/java-syntax-2026-09-24/generic-method-signatures/negative-fixtures/results.txt)一致。`GenericThrowsProbe` 的 `Signature <T:Number>(TT;)TT;` 没有 `^`，而 `Exceptions` 有 IOException，现接受；Object 界不匹配、未绑定和类级变量仍拒绝。复杂签名和正文不兼容类在 reader 层均通过，这是预期：完整拼写和正文类型证明属 2.3。`openspec validate recover-generic-method-signatures --strict`、`git diff --check` 通过。
