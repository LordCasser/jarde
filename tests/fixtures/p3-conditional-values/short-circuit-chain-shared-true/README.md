# Three-test shared-true field write

`ChainOrField.assign(ZZ)V` 是 `javac --release 8 -g:none` 对 `result = extra || left || rhs()` 的真实输出。两次外层 `ifne` 和内层条件共同到达 true 生产者 BCI 14；false 生产者在 BCI 18，BCI 19 唯一写入静态 `Z` 字段。`rhs()` 记录调用次数并返回可切换值，Runner 覆盖两条短路路径和 RHS 真/假路径。

从仓库根目录复核冻结产物；以下使用临时目录，不改写冻结 class：

```sh
FIXTURE=tests/fixtures/p3-conditional-values/short-circuit-chain-shared-true
TMP=$(mktemp -d /tmp/jarde-chain-or-freeze.XXXXXX)
javac --release 8 -g:none -d "$TMP" "$FIXTURE/ChainOrField.java"
cmp "$TMP/ChainOrField.class" "$FIXTURE/ChainOrField.class"
javac --release 8 -cp "$TMP" -d "$TMP" "$FIXTURE/Runner.java"
java -Xverify:all -cp "$TMP" Runner
javap -c -v -p "$TMP/ChainOrField.class"
```

原 class 输出：

```text
extra=true,left=false,rhs=false,result=true,calls=0
extra=false,left=true,rhs=false,result=true,calls=0
extra=false,left=false,rhs=true,result=true,calls=1
extra=false,left=false,rhs=false,result=false,calls=1
```

SHA-256：

- `ChainOrField.java`: `322c2853b6a243557fb7a089f6052c5cbcf846a703d7adc41b899ef32ab3b4c2`
- `ChainOrField.class`: `30400aee839ed464b8ad804acc56cd99d382c5e8300c1e39ec4bd00632290666`
- `Runner.java`: `b0e6044878d5d2ced0e03b76a54d00eaafeaa318d0c43b94874faf7539663a6e`
