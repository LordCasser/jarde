# CF-10 两个隔离缺口的主线复核

root 在合入 CF-14 后的主线用 CLI SHA-256 `0c299a1c0143259d5cfe2580ad2ba30e101168ba5c1cec972874952a366086f4` 独立从两份归档原 class 生成 Jarde 完整源码。步长索引样本 class SHA-256 `64b5baa6270ce4604c0a70fa0a6d3438902bdeaa9f3bc3f7cb5105b40c512344`，Jarde 源码 SHA-256 `37ae8a2bc053dbd22486266b29de686777eb21fb459ea5df29b3db2e9e514137`；`List→Iterable` 调用样本分别为 `bbae590902f24b48fa064c25b59247ec71fcb1534ec0818a54ce437b5f1ba0ce` 与 `32d7fa4b815ec49d5c9d9c6ea3d8a1087a1e910325f1664e6d736f78739dc205`。两份重生 Jarde 源码均与审计归档逐字节相同。

root 重新以 `javac --release 8 -g -Xlint:-options` 编译完整原/JADX/Jarde 类并运行 `java -Xverify:all`。步长索引原/JADX 均输出 `4`，Jarde 因 `everyOther` 缺返回不能编译；调用样本三方均能编译，原/JADX 输出 `called`，Jarde 没有输出。这确认是两个独立缺口：前者的结构/局部绑定尚未恢复，后者在没有任何循环或重载时仍漏执行物理调用。后者的[窄 OpenSpec](../../../../changes/preserve-list-iterable-invocation-widening/)只处理调用参数证据；CF-10 仍是部分已测单元。
