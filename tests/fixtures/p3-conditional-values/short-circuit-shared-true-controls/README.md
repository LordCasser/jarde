# Shared-true rejection controls

本目录固定两个拒绝控制。`SharedTrueDuplicatePhi.assign(Z)V` 是 Java 8 `result = other = left || rhs()`，在单个 Phi 后执行两个 `putstatic`，用于检验重复消费者。`SharedTrueNonBoolean.class` 来自已有冻结单写入类 `short-circuit-shared-true/SharedTrueShortCircuit.class`：只把 BCI 10 的 `iconst_1` 改成 `iconst_2`。这使非布尔生产者拒绝与第二消费者拒绝互相独立。补丁不改变 StackMap 中的 `int` 类型，且由 JVM 验证器接受。

从仓库根目录重建控制 class 和 runner：

```sh
FIXTURE=tests/fixtures/p3-conditional-values/short-circuit-shared-true-controls
BASE=tests/fixtures/p3-conditional-values/short-circuit-shared-true
TMP=$(mktemp -d /tmp/jarde-shared-true-controls.XXXXXX)
mkdir -p "$TMP/classes" "$TMP/nonboolean"
javac --release 8 -g:none -d "$TMP/classes" "$BASE/SharedTrueShortCircuit.java" "$FIXTURE/SharedTrueDuplicatePhi.java"
javac --release 8 -g:none -cp "$TMP/classes" -d "$TMP/classes" "$FIXTURE/VerifySharedTrueControls.java"
cp "$TMP/classes/SharedTrueDuplicatePhi.class" "$FIXTURE/SharedTrueDuplicatePhi.class"
python3 -c 'from pathlib import Path; import sys; p=Path(sys.argv[1]); b=bytearray(p.read_bytes()); old=bytes.fromhex("04 a7 00 04 03 b3 00 11"); assert b.count(old)==1; i=b.index(old); b[i]=0x05; Path(sys.argv[2]).write_bytes(b)' "$TMP/classes/SharedTrueShortCircuit.class" "$FIXTURE/SharedTrueNonBoolean.class"
```

用 `javap` 检查 BCI 和 StackMap，再用完整 JVM 验证运行两种 class：

```sh
javap -classpath "$TMP/classes" -c -v SharedTrueDuplicatePhi
cp "$FIXTURE/SharedTrueNonBoolean.class" "$TMP/nonboolean/SharedTrueShortCircuit.class"
javap -classpath "$TMP/nonboolean" -c -v SharedTrueShortCircuit | sed -n '/static void assign/,/stack = \[ int \]/p'
java -Xverify:all -cp "$TMP/classes" VerifySharedTrueControls
java -Xverify:all -cp "$TMP/nonboolean:$TMP/classes" VerifySharedTrueControls
```

未补丁版本输出 `duplicate-true:true,0`、`duplicate-false:true,1`、`single-true:true,0`、`single-false:true,1`。补丁版的两行单写入输出变为 `single-true:false,0`、`single-false:false,1`；这符合 `iconst_2` 存入 JVM `Z` 字段时保留最低位的语义。两次运行均通过 `java -Xverify:all`。

SHA-256：

- `SharedTrueDuplicatePhi.java`: `6ba97d801f516bb828105c10856cef5805fb3aa81ebaf18061d3d4642c2ba7ce`
- `SharedTrueDuplicatePhi.class`: `ec4ff7912cc1700eebf93b831c70f48a031617707535c9bbbb110ac6eaa9457e`
- `SharedTrueNonBoolean.class`: `7ac7559fa925eae32e3ee7493e8408b7a290d3b4176d7f5cbfe2c992815d5144`
- `VerifySharedTrueControls.java`: `94f6ccf4ef2c8c814470b73472ff034ddae683b79ab00d85e491753fae8db984`
