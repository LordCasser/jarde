# 注解头修后独立整类重放

root 在原 `header-minimal/` fixture 的临时复制上执行 `run_audit.py`，只把脚本的旧 CLI SHA pin 替换成最终构建的冻结 CLI SHA `7a33dbed5b390009cb65802fd9264367e1d5854b9cc70dbced5ae1878109c822`，并以 `/tmp/jarde-cli-syntax-root-final` 作为 `JARDE_CLI`。脚本重新编译原源码、运行原 class、重新由 JADX 生成并编译完整类、再由 Jarde 生成并编译完整类。原源码、class SHA 清单及原/JADX 运行文本与修前夹具逐字节相同；CLI 哈希在重放前后相同。

`basic/` 的原/JADX/Jarde 三方均在 Java 8 编译并经 `java -Xverify:all` 反射执行，输出同为 `5`、`source-only`、`[2, 4]`。Jarde 的完整 `Basic.java` 写为 `@interface Basic`，结构化 JSON 仍保留原始 Annotation 接口表。

`nested/` 的 `Inner` 和 `Nested` 头都成为合法 `@interface`，完整 Jarde 类能通过 javac；但 `Nested.child/children` 默认值尚未恢复，runner 在 `getDefaultValue()` 后抛出 NPE，原/JADX 则输出 `6`、`2`。这是 `recover-nested-annotation-defaults` 的独立 RED，不能因源码能编译而宣称语义已相同。具体 class SHA、生成文本、JSON、编译/执行结果见 `summary.json` 和各 case 文件。
