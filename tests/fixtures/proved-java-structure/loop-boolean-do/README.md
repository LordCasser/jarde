# 复合 do-while 测试

`DoLoopBool.andDo` 与 `DoLoopBool.orDo` 的循环体相同，只把闩锁上的两个测试分别接成 `&&`、`||`。冻结 class 是 `openspec/evidence/java-syntax-2026-09-26/loop-boolean-do/analysis.md` 三方对照使用的 Java 8 输入。

源码 `DoLoopBool.java` 的 SHA-256 为 `4d2ef462741a2d866cb24e729023730534c19249a7cf447b49d4a86135ff56da`；冻结 `DoLoopBool.class` 为 `900d52f6329d4ce5240f0cf857a847e791aacf83a4b4d99988f72248b0de9c40`。

在仓库根目录运行 `./tests/fixtures/proved-java-structure/loop-boolean-do/run.sh`。它以 `javac --release 8 -g:none` 重编，并用 `java -Xverify:all` 执行三组有界输入；预期输出为：

```text
3,3:3,3
3,-1:1,3
0,0:1,1
```

只冻结 `DoLoopBool.class`；`Runner.class` 仅由 `run.sh` 在临时目录生成。重新生成冻结输入：

```sh
javac --release 8 -g:none -d tests/fixtures/proved-java-structure/loop-boolean-do tests/fixtures/proved-java-structure/loop-boolean-do/DoLoopBool.java
```
