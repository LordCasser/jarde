# 重载参数类型边界证据

本目录记录两个只读架构探针。源码均为本目录自写样例，使用 `javac 23.0.1 --release 8 -g:none`；
反编译器为 `jadx 1.5.6`。jarde 使用实验时已有的
`target/debug/jarde-cli`，SHA-256 为
`3c8a06e0a300787ca457802cb0024a01ed681e6a87137febfff0d1b13c5d2a19`。本次没有运行 cargo，也没有修改
production Rust、spec 或 tasks。

## GenericFactory 的 Object 重载

`GenericOverloadProbe.genericObjectCast` 的源码是：

```java
return choose((Object) GenericFactory.make());
```

`GenericFactory.make` 的擦除返回描述符是 `()Ljava/lang/Object;`，原 class 的调用池项也是
`choose(Ljava/lang/Object;)I`。原程序输出 `genericObjectCast=1`。jarde 和 jadx 都去掉了源码中
限定泛型调用的 `(Object)`，生成 `choose(GenericFactory.make())`；这份文本可以编译，但 Java 的
泛型目标类型推断把调用选成 `choose(String)`，两者都执行为 `genericObjectCast=2`。

因此“呈现表达式类型已经是 Object”不等于源码重载选择已经固定为 Object：泛型 invocation 是
poly expression，外层重载仍会参与它的目标类型推断。对 Object 目标补 `(Object)` 不会新增运行时
失败（包括 null），但不能把这个结论推广到任意接口或具体类目标。

## 去掉 Runnable checkcast 的 JVM 边界

`InterfaceBoundaryProbe.caller` 的原始字节码为 `aload_0; checkcast Runnable; invokestatic take;
ireturn`。`patch_interface_checkcast.py` 只把这条 checkcast 的三个字节替换为三个 `nop`，不改
常量池、方法描述符或代码长度。`java -Xverify:all` 接受 patched class，调用
`caller(new Object())` 输出 `object=7`；原 class 则在 checkcast 处以 `ClassCastException` 退出。

这说明 JVMS verifier 对此处的 interface 参数允许栈上的 `Object` 进入 `take(Runnable)`，而运行时
并没有自动补检查。仅凭 callee descriptor 合成 `(Runnable)` 会把合法的 patched class 改成抛
`ClassCastException` 的程序，因此未知 reference subtype 应保守拒绝或保留 fallback；只有 class
file 自己给出 checkcast，或目标类型是不会失败的安全上溯（如 Object）时，才可讨论补静态类型。

对同一个 patched class，jarde 保留 BCI 1/2/3 的 fallback 诊断，但周围仍写出
`return take(arg0);`；这份恢复文本因 `Object` 不能直接传给 `Runnable` 而无法 javac。jadx 输出
同样是 `return take(obj);` 并带有 type inference warning，也无法 javac。这里的编译失败是当前
恢复层的边界证据，不能用修过的方法体替代原 class 执行结果。

## 文件与重放

`generic-*` 保存 GenericFactory 探针的 javap、jarde/jadx 文本、编译日志与三份输出；
`interface-*` 保存原/patch 后 javap、patch 日志、JVM 输出、jarde/jadx 文本和编译失败日志。
为让默认包 fixture 可直接重编译，保存的 `*-jadx.java.txt` 已去掉 jadx 自动添加的
`package defpackage;`，其余类体保持原样。
原始与 patched class 只放在 `/tmp/jarde-type-boundaries-20260923/`，可由脚本重建。

```sh
E=openspec/evidence/java-syntax-2026-09-22/overloads/type-boundaries
T=/tmp/jarde-type-boundaries-20260923
mkdir -p "$T/generic/original" "$T/interface/original" "$T/interface/patched"
javac --release 8 -g:none -d "$T/generic/original" \
  "$E/GenericFactory.java" "$E/GenericOverloadProbe.java" "$E/GenericOverloadRunner.java"
java -cp "$T/generic/original" GenericOverloadRunner
javap -classpath "$T/generic/original" -p -c -s GenericOverloadProbe

javac --release 8 -g:none -d "$T/interface/original" \
  "$E/InterfaceBoundaryProbe.java" "$E/InterfaceBoundaryRunner.java"
java -Xverify:all -cp "$T/interface/original" InterfaceBoundaryRunner
python3 "$E/patch_interface_checkcast.py" \
  "$T/interface/original/InterfaceBoundaryProbe.class" \
  "$T/interface/InterfaceBoundaryProbe.nocheckcast.class"
cp "$T/interface/InterfaceBoundaryProbe.nocheckcast.class" \
  "$T/interface/patched/InterfaceBoundaryProbe.class"
cp "$T/interface/original/InterfaceBoundaryRunner.class" "$T/interface/patched/"
java -Xverify:all -cp "$T/interface/patched" InterfaceBoundaryRunner
javap -p -c -s "$T/interface/InterfaceBoundaryProbe.nocheckcast.class"

target/debug/jarde-cli class-source \
  --input "$T/generic/original/GenericOverloadProbe.class" \
  --class GenericOverloadProbe --policy single-class --release 8 --format text
target/debug/jarde-cli class-source \
  --input "$T/interface/InterfaceBoundaryProbe.nocheckcast.class" \
  --class InterfaceBoundaryProbe --policy single-class --release 8 --format text
jadx --no-res -d "$T/generic/jadx" "$T/generic/original/GenericOverloadProbe.class"
jadx --no-res -d "$T/interface/jadx" \
  "$T/interface/InterfaceBoundaryProbe.nocheckcast.class"

mkdir -p "$T/generic/jarde" "$T/generic/jadx-run"
cp "$E/generic-jarde.java.txt" "$T/generic/jarde/GenericOverloadProbe.java"
cp "$E/generic-jadx.java.txt" "$T/generic/jadx-run/GenericOverloadProbe.java"
cp "$E/GenericFactory.java" "$E/GenericOverloadRunner.java" "$T/generic/jarde/"
cp "$E/GenericFactory.java" "$E/GenericOverloadRunner.java" "$T/generic/jadx-run/"
javac --release 8 -g:none -d "$T/generic/jarde" "$T/generic/jarde/"*.java
java -cp "$T/generic/jarde" GenericOverloadRunner
javac --release 8 -g:none -d "$T/generic/jadx-run" "$T/generic/jadx-run/"*.java
java -cp "$T/generic/jadx-run" GenericOverloadRunner

# The patched interface text intentionally remains a fallback/non-Java artifact: javac should
# report Object -> Runnable for both jarde and jadx outputs, while the patched class above runs.
```
