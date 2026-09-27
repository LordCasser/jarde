EM-20 的正例源码是 [`openspec/evidence/java-syntax-2026-09-27/em20-local-scopes/input/em20/LocalScopes.java`](../../../openspec/evidence/java-syntax-2026-09-27/em20-local-scopes/input/em20/LocalScopes.java)。负例源码为同目录的 `NonGuard.java`。以 Java 8 无 debug 信息冻结：

```sh
javac --release 8 -g:none -d /tmp/em20-fixture-classes openspec/evidence/java-syntax-2026-09-27/em20-local-scopes/input/em20/LocalScopes.java tests/fixtures/em20-monitor-reuse/NonGuard.java
cp /tmp/em20-fixture-classes/em20/LocalScopes.class tests/fixtures/em20-monitor-reuse/LocalScopes.class
cp /tmp/em20-fixture-classes/NonGuard.class tests/fixtures/em20-monitor-reuse/NonGuard.class
```

SHA-256：`LocalScopes.class` 为 `25386b1b3b3aed6c95d74dc44d4d2ee3703f36f66897c8a370e2d93577ff4991`；`NonGuard.class` 为 `b87257948a1169c6d3c102d78e73b559d469bf863dd744cd5c1d15d23fb5824d`。正例的 BCI 3/14 引用槽位在 BCI 20/22 变为循环整数，异常表的两行均属已证 monitor 清理。负例分别有普通 catch 异常边、后续 handler 对复用整数的读取、后续循环回到同步 guard 的正常边；三者都不能借用正例的分割证明。字节码细节以 `javap -c -p` 可复核。
