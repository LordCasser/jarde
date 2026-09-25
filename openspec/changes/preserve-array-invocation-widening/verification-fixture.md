# 数组调用上溯 fixture 验证

## 冻结输入

永久输入在 `tests/fixtures/p3-array-invocation-widening/`：

- `ArrayReferenceCore` 是无数组写入的最小完整正例，Java 8 class 为 1,632 B、16 个 `Code` 属性，SHA-256 `549e371ac98362c3ca0d0c992e7a7cb2292a90a5f9b46ac58ee468338dd9ae0d`。它覆盖三种需固定的调用参数（`int[][] → Object[]`、`String[] → Object[]`、`String[][] → Object[][]`）、既有 `Object`/`Object[]` 目标、运行时数组类、空数组和 null；runner 输出 28 行。
- `ArrayReferenceOverloadTarget` 让 `localObjectTarget` 先把 `String[]` 存入 `Object[]` 局部变量，再调用同时含 `overload(Object[])` 与 `overload(String[])` 的重载组；该方法的真实 Methodref 是 `overload([Ljava/lang/Object;)`，局部上溯没有 `checkcast`。runner 共 8 行，比较隐式 `String[]` 重载、null/零长数组与一次求值副作用。
- `ArrayInterfaceProbe`（1,038 B，SHA-256 `83f4f36a7f20e4ee12e2d8458a396746da78682076bd059eaa45f6c8fc4a3580`）固定 JLS 8 的内置数组 marker 关系：`int[][] → Cloneable[]`、`String[][] → Serializable[]`、`int[] → Cloneable`。原 class、JADX 和恢复类的三行输出必须一致。
- `UnknownArrayRelation` 的嵌套类实现合法 Java 源码，但验证时只把外层 class 提供给恢复器。因用户类继承和用户接口实现关系不可见，`Child[] → Base[]` 与 `MarkedChild[] → UserMarker[]` 应保留完整拒绝。

所有 subject 的冻结 class 在 `tests/fixtures/p3-array-invocation-widening/v8/`，总计 4,450 B。runner 和未知关系的嵌套 class 只保留源码；验证时在临时目录按 `--release 8` 编译。完整 37 行的 `ArrayReferenceConversions` 仍留在历史审计证据中：其额外 `new Object()`/数组写入缺口独立于本 change，不能作为本 change 的上溯验收输入。

## 修复前基线

基线审计使用冻结 CLI `/tmp/jarde-cli-deferred-final-ecab`（SHA-256 `ecab8244d1709765fa0f2b0effcebe49b9f066eea911843d11d34abec5837330`）。为不改写历史证据目录，把审计副本放在临时目录：

```sh
audit_tmp=$(mktemp -d /tmp/array-widen-audit.XXXXXX)
cp -R openspec/evidence/java-syntax-2026-09-22/array-reference-conversion "$audit_tmp/array-reference-conversion"
python3 "$audit_tmp/array-reference-conversion/run_audit.py"
cat "$audit_tmp/array-reference-conversion/summary.json"
```

重放结果与冻结证据相同：core 原 class 和 JADX 都成功编译、通过 `java -Xverify:all`，28 行输出逐字节相同；Jarde 对三个协变调用各有一个拒绝，core 完整生成源码 `javac --release 8` 失败。重载目标原 class 与 JADX 都通过 Java 8 编译和验证执行，8 行输出相同；旧 Jarde 对无 `checkcast` 的局部 `Object[]` 调用拒绝，完整源码不能编译。新增 marker 原 class 与 JADX 同样通过编译和 JVM 验证，3 行输出相同；冻结 Jarde 对三个 marker 调用各有一个拒绝，生成源码在三处缺少返回语句而编译失败。37 行完整样本也有 core 的三个拒绝；其另有数组写入拒绝，不纳入本案正例。

## 实施后重放

将 `JARDE_BIN` 指向本次构建的 CLI 二进制。以下步骤在一个临时目录里重新编译原类、JADX 类和 Jarde 类，并用同一个 runner 比较完整执行输出；`set -e` 使任一步失败即停止：

```sh
fixture=tests/fixtures/p3-array-invocation-widening
tmp=$(mktemp -d /tmp/array-widen-verify.XXXXXX)
trap 'python3 -c "import shutil,sys; shutil.rmtree(sys.argv[1])" "$tmp"' EXIT

mkdir -p "$tmp/original" "$tmp/jarde" "$tmp/jadx" "$tmp/jadx-classes"
javac --release 8 -g:none -d "$tmp/original" \
  "$fixture/ArrayReferenceCore.java" "$fixture/ArrayReferenceCoreRunner.java"
test "$(shasum -a 256 "$tmp/original/ArrayReferenceCore.class" | cut -d' ' -f1)" = \
  549e371ac98362c3ca0d0c992e7a7cb2292a90a5f9b46ac58ee468338dd9ae0d
java -Xverify:all -cp "$tmp/original" ArrayReferenceCoreRunner > "$tmp/original-core.txt"
diff -u "$fixture/expected-core.txt" "$tmp/original-core.txt"

"$JARDE_BIN" class-source --input "$tmp/original/ArrayReferenceCore.class" \
  --class ArrayReferenceCore --policy single-class --release 8 --format text \
  > "$tmp/jarde/ArrayReferenceCore.java"
javac --release 8 -g:none -d "$tmp/jarde" \
  "$tmp/jarde/ArrayReferenceCore.java" "$fixture/ArrayReferenceCoreRunner.java"
java -Xverify:all -cp "$tmp/jarde" ArrayReferenceCoreRunner > "$tmp/jarde-core.txt"
diff -u "$tmp/original-core.txt" "$tmp/jarde-core.txt"

jadx --no-res -d "$tmp/jadx" "$tmp/original/ArrayReferenceCore.class"
# JADX 把默认包放进 defpackage；这里只机械删除该 package 行以编译原 runner。
sed '/^package defpackage;$/d' \
  "$tmp/jadx/sources/defpackage/ArrayReferenceCore.java" \
  > "$tmp/jadx/ArrayReferenceCore.java"
javac --release 8 -g:none -d "$tmp/jadx-classes" \
  "$tmp/jadx/ArrayReferenceCore.java" "$fixture/ArrayReferenceCoreRunner.java"
java -Xverify:all -cp "$tmp/jadx-classes" ArrayReferenceCoreRunner > "$tmp/jadx-core.txt"
diff -u "$tmp/original-core.txt" "$tmp/jadx-core.txt"
```

对 `ArrayReferenceOverloadTarget` 和 `ArrayInterfaceProbe` 重复相同的三阶段流程，分别使用配套 runner 与 `expected-overload-target.txt`、`expected-array-interfaces.txt`；JADX 仅作同样的默认包行标准化，不修改任何生成方法。用 `javap -c -p` 检查 `localObjectTarget` 的实际 `invokestatic overload:([Ljava/lang/Object;)`，恢复源码必须对 `Object[]` 目标保留显式静态类型；runner 的 `effectful-evaluations=1` 和数组运行时类、长度结果也必须相同。

最后用 `ArrayReferenceOverloadTarget.class` 与 `ArrayInterfaceProbe.class` 分别单类驱动恢复。对 `UnknownArrayRelation.class` 只提交外层 class，检查两个调用仍含来源完整的 `no safe reference conversion evidence` 拒绝；原源码 runner 的输出应等于 `expected-unknown-relation.txt`。`int[] → Object[]` 是非法 Java 调用，必须在纯数组关系判据测试中拒绝，不制造伪 class 输入。
