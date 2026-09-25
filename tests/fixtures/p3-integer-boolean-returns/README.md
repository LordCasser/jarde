# Java 8 整数返回 boolean fixture

`IntegerBooleanReturns.java` 只写 Java 8 可编译的 `int` 返回方法。`regenerate.py` 用
`javac --release 8 -g:none` 编译后，只改 `direct`、`once`、`post`、`pre`、`sync`、`syncOn`
六个方法的返回 descriptor 为 `Z`；方法 Code 保持原样。补丁 JSON 同时记录输入/输出 class
SHA-256 和补丁前后 Code SHA-256。`IntegerBooleanReturnsRunner.java` 在 `-Xverify:all` 下验证
直接返回、单次调用计数、字段自增完整值、监视器返回与 null 监视器异常，输出冻结于
`v8/expected.txt`。

共享栈边界从已验证的 major 49 `ActualStackJoin` byte/stack-join class 复制，只把
`runByte(I)B` 的返回描述符改成 `runByte(I)Z`，不改 Code；`ireturn` 是两 switch 臂共享的真实
返回位置。该 class 保留 major 49 及旧 class 的 switch 结构，不将条件 stack phi 当作已恢复
正例。对应的精确 descriptor 补丁、hash 与 JVM 输出分别在 `v8/actual-stack-join/`。

复现：

```sh
python3 tests/fixtures/p3-integer-boolean-returns/regenerate.py /tmp/jarde-intbool-gen
```
