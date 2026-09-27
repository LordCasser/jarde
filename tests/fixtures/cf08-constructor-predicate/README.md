# CF-08 固定类与 verifier 有效拒绝样本

`cf08/NotIndexedLoop.class` 由 `openspec/evidence/java-syntax-2026-09-27/cf08-endless-loops/input/cf08/NotIndexedLoop.java` 以 `javac --release 8 -g:none` 重建，SHA-256 为 `b1e6bd92f8fe92e726e7a2a95461327dd3f7abe65ee7f451f1225777ed7ea6d4`。它是固定 JADX `TestNotIndexedLoop` 的**完整类**，含原构造器和 `test` 方法。

`NotIndexedLoopNegatives.java` 从同一完整方法逐项改变构造器结果转送/额外消费、构造参数效果、谓词接收者/调用/效果、内层额外出口、内层 join 效果或外层 null 输入。所有负例都由 Java 8 编译器产生 verifier 有效的 class；它们测试新证书的拒绝边界，而不要求 JVM 接受未初始化对象或未定义局部的非法字节码。

从仓库根目录重建并运行：

```sh
javac --release 8 -g:none -Xlint:-options -d tests/fixtures/cf08-constructor-predicate \
  openspec/evidence/java-syntax-2026-09-27/cf08-endless-loops/input/cf08/NotIndexedLoop.java \
  tests/fixtures/cf08-constructor-predicate/cf08/NotIndexedLoopNegatives.java \
  tests/fixtures/cf08-constructor-predicate/cf08/VerifierRunner.java
java -Xverify:all -cp tests/fixtures/cf08-constructor-predicate cf08.VerifierRunner
```

预期九行按 Runner 中 `names` 顺序为 `f / f / f / f / f / f / f / f / x`。`p3_effectful_exits` 测试逐方法核对这些 class 保留物理引用且不产生受证的 `while (true)`。已有双出口及二层三来源样本继续由同文件的原测试覆盖。
