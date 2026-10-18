# Indexed List 参数声明控制

原始构造源与 class 均来自 root 实际基线 v2；class 为 JDK23 `--release 8 -proc:none` 的完整 Java 8 产物，分别保留与去除 debug 信息。没有手改 class 或反编译生成源码。

void、boolean、int、long 正例保留真实 String 转换，其中 int/long 直接使用 get 结果。错 Integer 转换、参数重写与逃逸、raw/wildcard List、引用返回作为不误准入的控制。void 使用 `value.length()`，避免独立的 loop 内显式 throw 恢复缺口遮住本片；旧 if-null-throw 源及全类失败已完整保留在 OpenSpec 结果中。

完整原始双编译器/双 JVM、Jarde/JADX 原始流、Code/BCI 与事件/异常/CodeSource 核验见 `openspec/changes/recover-proved-indexed-list-parameter-signatures/results/controls-baseline-root-v1/`。旧 CLI56 的 Jarde 16次完整重编/32次VM/512case同原；4次JADX完整重编因 String→Integer 直接转换不合法而失败，不能当runtime通过。本README记录基线，不冒称候选实现已验收。

源码 SHA256：`d27ec67a11e408edb4d1a19b7a9b9bc2f72bdb6528798425a107afd73d3f048c`。

```json
[
  {
    "profile": "debug",
    "bytes": 2970,
    "sha256": "d71671a2f7f73edf3aada8420c3bdda73e638d3f92b7c6489f0d2da25dfdb2cb",
    "compiler_argv": [
      "/Users/lordcasser/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home/bin/javac",
      "--release",
      "8",
      "-g",
      "-encoding",
      "UTF-8",
      "-proc:none",
      "-classpath",
      "/private/tmp/jarde-tf3-original-inputs-root-v1-20261018/jadx-test-harness.jar:/Users/lordcasser/workspace/testzone/jadx/jadx-core/build/classes/java/main:/Users/lordcasser/workspace/testzone/jadx/jadx-plugins/jadx-input-api/build/classes/java/main:/Users/lordcasser/workspace/testzone/jadx/jadx-commons/jadx-zip/build/classes/java/main:/Users/lordcasser/workspace/testzone/jadx/jadx-commons/jadx-analysis/build/classes/java/main:/private/tmp/jarde-tf3-original-harness-deps-root-v1-20261018/annotations-26.1.0.jar:/private/tmp/jarde-tf3-original-harness-deps-root-v1-20261018/assertj-core-3.27.7.jar:/private/tmp/jarde-tf3-original-harness-deps-root-v1-20261018/junit-jupiter-api-5.14.4.jar:/private/tmp/jarde-tf3-original-harness-deps-root-v1-20261018/junit-platform-commons-1.14.4.jar:/Users/lordcasser/.m2/repository/org/apiguardian/apiguardian-api/1.1.2/apiguardian-api-1.1.2.jar:/Users/lordcasser/.m2/repository/org/opentest4j/opentest4j/1.3.0/opentest4j-1.3.0.jar",
      "-sourcepath",
      "/private/tmp/jarde-testnestedloops2-controls-baseline-root-v2-20261019/empty-classpath",
      "-d",
      "/private/tmp/jarde-testnestedloops2-controls-baseline-root-v2-20261019/debug/classes",
      "/private/tmp/jarde-testnestedloops2-controls-baseline-root-v2-20261019/source/jadx/tests/integration/loops/NestedLoops2GenericControls.java"
    ]
  },
  {
    "profile": "no-debug",
    "bytes": 1891,
    "sha256": "860d037a221b4444d8024d46475b4f7ec3c77d941800998b15fa58b96385c09a",
    "compiler_argv": [
      "/Users/lordcasser/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home/bin/javac",
      "--release",
      "8",
      "-g:none",
      "-encoding",
      "UTF-8",
      "-proc:none",
      "-classpath",
      "/private/tmp/jarde-tf3-original-inputs-root-v1-20261018/jadx-test-harness.jar:/Users/lordcasser/workspace/testzone/jadx/jadx-core/build/classes/java/main:/Users/lordcasser/workspace/testzone/jadx/jadx-plugins/jadx-input-api/build/classes/java/main:/Users/lordcasser/workspace/testzone/jadx/jadx-commons/jadx-zip/build/classes/java/main:/Users/lordcasser/workspace/testzone/jadx/jadx-commons/jadx-analysis/build/classes/java/main:/private/tmp/jarde-tf3-original-harness-deps-root-v1-20261018/annotations-26.1.0.jar:/private/tmp/jarde-tf3-original-harness-deps-root-v1-20261018/assertj-core-3.27.7.jar:/private/tmp/jarde-tf3-original-harness-deps-root-v1-20261018/junit-jupiter-api-5.14.4.jar:/private/tmp/jarde-tf3-original-harness-deps-root-v1-20261018/junit-platform-commons-1.14.4.jar:/Users/lordcasser/.m2/repository/org/apiguardian/apiguardian-api/1.1.2/apiguardian-api-1.1.2.jar:/Users/lordcasser/.m2/repository/org/opentest4j/opentest4j/1.3.0/opentest4j-1.3.0.jar",
      "-sourcepath",
      "/private/tmp/jarde-testnestedloops2-controls-baseline-root-v2-20261019/empty-classpath",
      "-d",
      "/private/tmp/jarde-testnestedloops2-controls-baseline-root-v2-20261019/no-debug/classes",
      "/private/tmp/jarde-testnestedloops2-controls-baseline-root-v2-20261019/source/jadx/tests/integration/loops/NestedLoops2GenericControls.java"
    ]
  }
]
```
