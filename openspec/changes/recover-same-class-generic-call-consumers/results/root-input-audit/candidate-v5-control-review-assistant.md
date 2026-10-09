# CLI5 控制证据复核辅助页（不作新裁定）

本页只把既有 CLI3/CLI4 root disposition 与 CLI5 保存的 44 条逐腿事实并排，供 root 复核。CLI3/CLI4 的历史 verdict/basis 保持原样；本页不生成、不修改 CLI5 root verdict，也不把编译或 Probe 成功当作拒绝边界正确。

## 身份与比较结果

- CLI3 facts SHA-256 `72c270bf5b0c81de5c4c70a8d1ba3f99aeb9dba9300e8fa558db2c21ef82a334`；CLI3 root disposition SHA-256 `e5f852255ee9bcb36f9eeadf413729aedaddfeb6150bb66d425d43a4b12c5e8c`。
- CLI4 facts SHA-256 `976133b57d131de9d187cc9c1dd2cd70d081beaa87855613e4de0d6ac4518a36`；CLI4 root disposition SHA-256 `ea331f1e575e095911450a4cb5a064014639f901f5bb500bdfad7525c2b24982`。CLI4 disposition 记录 `v3_v4_actual_sources_and_probe_identical=true`。
- CLI5 CLI SHA-256 `010e9f1ecd82a1f2c7c4dcd49010afc787d3980a0194e9fc6a1d12994132dac5`；CLI5 facts SHA-256 `fbc51c9c9f7e8b54f8dff4978330bc0399bb004c4c4dd55e2c533068e9dcc0d4`；冻结源码身份 v36 archive SHA-256 `870589b655d855c7eedf6951c230bd8a69158f98b4ea54f54c14da008d9dc9a2`。
- 固定 acceptance runner SHA-256 `a19895e70ef306a3024132a3da7c07cf891ea87e5ef0e25e01413989514ce836`；run manifest SHA-256 `15e1494d3414693aad8bfa7c3d50b4f3cbdb58f93e21b880fb7183b7725ab130`；144 行、integrity_errors=0。
- 44/44 输入源码、输入 jar、Probe source hash 在 CLI3/4/5 相同；CLI3 与 CLI4 的 candidate source hash/Probe stdout 对 44/44 相同。CLI5 对比 CLI4，candidate source bytes 在 36/44 行改变；类/方法声明头对比 44/44 不变，candidate Probe stdout 与行为对比 44/44 不变。
- CLI5 固定判据：七项 `CONTROL_REVIEW_ONLY` 均 `needs-root-review`；SameErasureBinder、MultiUseResult 是拒绝控制谓词各 4/4 pass；BridgeUnknown、IncompleteSite 是分开的正控制。完整矩阵仍是 `needs-review`。

## 范围和证据限制

逐腿事实源为 [candidate-v5-control-facts.json](candidate-v5-control-facts.json)，包含 44 个实际行、输入/输出 SHA-256、候选完整源码、Probe 转录和所保存的物理 javap 指令摘录。matrix/Nested 的独立 verifier 均已通过（只核身份、转录、输入、行为/反射数据及 isolation）。
以下“receiver”只能陈述输入 Java 源和 CLI5 展示源码上的表达式。保存的 CLI5 run 未包含内部 AST/SSA receiver 节点、绑定边、incoming-use worklist 或 proof object 的逐调用转录；所以 Java 展示源码、拒绝 marker、compile/Probe 不能证明实现内部如何归属 receiver，也不能单独证明闭包/拒绝判据正确。对 root basis 依赖此类内部事实的条目，这里明确标为证据未覆盖。

## 11 组、44 行逐项对照

### UnknownIncoming（GC-03）

- 历史 root disposition：CLI3/CLI4 均为 `bounded_refusal`，`full_api_recovered=False`。既有支持范围：另一入边参数是Object，原unchecked (T)源码cast擦除后不提供类型证明；三个关联头一起raw，独立untouched仍T。
- 固定输入源码 SHA-256 `23acc50b951ba101b524698c2788d5eb06500a4e827b2db2f1c59c63c5d8ec85`；原始源码实际文本：`public class UnknownIncoming<T> { public T identity(T x) { return x; } public T safe(T x) { return identity(x); } public T untouched(T x) { return x; } @SuppressWarnings("unchecked") public T unsafe(Object x) { return identity((T)x); } }
`
- 四腿实际输入 jar / CLI5 candidate source hash：
  - `corretto8/debug` jar `72327f51b077244ba5fe2896a7c8897acfd7e326ece2bc1c0c1fa1270bea519b`；source UnknownIncoming=8eb63a571e6f6f240b98d6cae458d58755233787208d41cc2b198b5804da7ef4
  - `corretto8/nodebug` jar `ea3309ef41feae4efe5091976a54899f25d1627ae4e572b7f5635669cb691577`；source UnknownIncoming=c13515486182a7c0233229cca698695bea4f7df50ff8f2e14b6fed5f0852e0ae
  - `openjdk23/debug` jar `dbe1f18fa7dcec4135024bfbe68d3c5a61d358a45ea69f63e2d0b20ecf314447`；source UnknownIncoming=8eb63a571e6f6f240b98d6cae458d58755233787208d41cc2b198b5804da7ef4
  - `openjdk23/nodebug` jar `6b1d92d63160c91655e838f87ba7ceccc5066dbba1bcaa01150fd6b4576cc2c3`；source UnknownIncoming=c13515486182a7c0233229cca698695bea4f7df50ff8f2e14b6fed5f0852e0ae
- CLI5 candidate 完整源 [UnknownIncoming.java](../candidate/candidate-v5/gc01-08-140/openjdk23/debug/UnknownIncoming/jarde/UnknownIncoming.java) SHA-256 `8eb63a571e6f6f240b98d6cae458d58755233787208d41cc2b198b5804da7ef4`；以下是实际保存源码摘录（行号为 candidate 文件行号）：
```java
1: // jarde: presentation of `UnknownIncoming` from the class file's own declaration and one recovery run per member.
2: // jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
3: // jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
4: public class UnknownIncoming<T> extends java.lang.Object {
5:     public UnknownIncoming() {
6:         // @method <init>()V
13:     // jarde: generic Signature projection refused for `identity(Ljava/lang/Object;)Ljava/lang/Object;`: same-class generic call dependency did not close over every incoming use
14:     public java.lang.Object identity(java.lang.Object x) {
15:         // @method identity(Ljava/lang/Object;)Ljava/lang/Object;
21:     // jarde: generic Signature projection refused for `safe(Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
22:     public java.lang.Object safe(java.lang.Object x) {
23:         // @method safe(Ljava/lang/Object;)Ljava/lang/Object;
26:         return this.identity(x);
29:     public T untouched(T x) {
31:         // @method untouched(Ljava/lang/Object;)Ljava/lang/Object;
37:     // jarde: generic Signature projection refused for `unsafe(Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
38:     public java.lang.Object unsafe(java.lang.Object x) {
39:         // @method unsafe(Ljava/lang/Object;)Ljava/lang/Object;
42:         return this.identity(x);
```
- 代表物理输入 jar `dbe1f18fa7dcec4135024bfbe68d3c5a61d358a45ea69f63e2d0b20ecf314447` 的 javap tool `/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home/bin/javap` SHA-256 `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e`；保存的输入字节摘录：
  - `1: invokespecial #1                  // Method java/lang/Object."<init>":()V`
  - `2: invokevirtual #7                  // Method identity:(Ljava/lang/Object;)Ljava/lang/Object;`
- CLI5 四腿 compile/Probe/behavior match 4/4；完整反射匹配 0/4；candidate behavior `behavior=safe+unchecked+independent`。这只是实测，不代替具体判据。
- CLI4→CLI5 实际 refusal marker 差异：
  - CLI4 only: `// jarde: generic Signature projection refused for \`safe(Ljava/lang/Object;)Ljava/lang/Object;\`: same-class generic call dependency did not close over every incoming use`
  - CLI4 only: `// jarde: generic Signature projection refused for \`unsafe(Ljava/lang/Object;)Ljava/lang/Object;\`: same-class generic call dependency did not close over every incoming use`
  - CLI5 only: `// jarde: generic Signature projection refused for \`safe(Ljava/lang/Object;)Ljava/lang/Object;\`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types`
  - CLI5 only: `// jarde: generic Signature projection refused for \`unsafe(Ljava/lang/Object;)Ljava/lang/Object;\`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types`
- 证据未覆盖/边界：候选 Object headers 与分项 marker 可见，但没有保存 incoming-edge/worklist/AST binding trace，不能证明 unchecked `(T)x` 的 Object 入边完整覆盖，也不能证明 safe 调用边被正确归类。

### CycleRelay（GC-08）

- 历史 root disposition：CLI3/CLI4 均为 `bounded_refusal`，`full_api_recovered=False`。既有支持范围：两个物理方法有互相invoke依赖，无环callee-first不以候选互证；有限执行控制保持行为但不计API恢复。
- 固定输入源码 SHA-256 `f68cd9f5d56020a7e03cbd263d7925770bce35e4bc8c57dd464a403eae1db905`；原始源码实际文本：`public class CycleRelay<T> { public T left(T x, boolean again) { if (again) return right(x,false); return x; } public T right(T x, boolean again) { if (again) return left(x,false); return x; } }
`
- 四腿实际输入 jar / CLI5 candidate source hash：
  - `corretto8/debug` jar `0a19c1468a138d4686dd223683ead772576d5527ff27b176b61aab00442eb4f2`；source CycleRelay=a0782cb01ebd2a66c1fa07fe758065f7d6b708dbc556ed72bd5276386f8ca2f3
  - `corretto8/nodebug` jar `93811aca27631b11d07b2be9f08e81612240e215f697b263633d2116e63cf808`；source CycleRelay=cd62d37698c910d793f9799686e71485b12029c2e87dcd3ffe3c3afe095b3f9b
  - `openjdk23/debug` jar `1436b39ebfabb2d3b47e1a42a0b0152b803c4d640f467afbad630b4f8c748b6e`；source CycleRelay=a0782cb01ebd2a66c1fa07fe758065f7d6b708dbc556ed72bd5276386f8ca2f3
  - `openjdk23/nodebug` jar `4724274f1301d83a75972e00b6071a02ab5b3f2d89f7a2fca84fde33273c369f`；source CycleRelay=cd62d37698c910d793f9799686e71485b12029c2e87dcd3ffe3c3afe095b3f9b
- CLI5 candidate 完整源 [CycleRelay.java](../candidate/candidate-v5/gc01-08-140/openjdk23/debug/CycleRelay/jarde/CycleRelay.java) SHA-256 `a0782cb01ebd2a66c1fa07fe758065f7d6b708dbc556ed72bd5276386f8ca2f3`；以下是实际保存源码摘录（行号为 candidate 文件行号）：
```java
1: // jarde: presentation of `CycleRelay` from the class file's own declaration and one recovery run per member.
2: // jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
3: // jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
4: public class CycleRelay<T> extends java.lang.Object {
5:     public CycleRelay() {
6:         // @method <init>()V
13:     // jarde: generic Signature projection refused for `left(Ljava/lang/Object;Z)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
14:     public java.lang.Object left(java.lang.Object x, boolean again) {
15:         // @method left(Ljava/lang/Object;Z)Ljava/lang/Object;
19:             return this.right(x, false);
25:     // jarde: generic Signature projection refused for `right(Ljava/lang/Object;Z)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
26:     public java.lang.Object right(java.lang.Object x, boolean again) {
27:         // @method right(Ljava/lang/Object;Z)Ljava/lang/Object;
31:             return this.left(x, false);
```
- 代表物理输入 jar `1436b39ebfabb2d3b47e1a42a0b0152b803c4d640f467afbad630b4f8c748b6e` 的 javap tool `/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home/bin/javap` SHA-256 `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e`；保存的输入字节摘录：
  - `1: invokespecial #1                  // Method java/lang/Object."<init>":()V`
  - `7: invokevirtual #7                  // Method right:(Ljava/lang/Object;Z)Ljava/lang/Object;`
  - `7: invokevirtual #13                 // Method left:(Ljava/lang/Object;Z)Ljava/lang/Object;`
- CLI5 四腿 compile/Probe/behavior match 4/4；完整反射匹配 0/4；candidate behavior `behavior=finite-cycle-marker`。这只是实测，不代替具体判据。
- CLI4→CLI5 实际 refusal marker 差异：
  - CLI4 only: `// jarde: generic Signature projection refused for \`left(Ljava/lang/Object;Z)Ljava/lang/Object;\`: same-class generic call dependency did not close over every incoming use`
  - CLI4 only: `// jarde: generic Signature projection refused for \`right(Ljava/lang/Object;Z)Ljava/lang/Object;\`: same-class generic call dependency did not close over every incoming use`
  - CLI5 only: `// jarde: generic Signature projection refused for \`left(Ljava/lang/Object;Z)Ljava/lang/Object;\`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types`
  - CLI5 only: `// jarde: generic Signature projection refused for \`right(Ljava/lang/Object;Z)Ljava/lang/Object;\`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types`
- 证据未覆盖/边界：输入物理互相 invoke 和 candidate Object header/marker 可见；未保存推断依赖图或 cycle/SCC rejection trace。

### MethodHandleUse（GC-08）

- 历史 root disposition：CLI3/CLI4 均为 `bounded_refusal`，`full_api_recovered=False`。既有支持范围：身份目标位于bootstrap handle，caller没有普通direct identity invoke；Function.apply跨接口的代换不在此片。
- 固定输入源码 SHA-256 `277e6d2ad33e55ee5774345acf929e9ee9a3c6a4694604acfa28a3676be1d8e0`；原始源码实际文本：`public class MethodHandleUse<T> { public T identity(T x) { return x; } public T relay(T x) { java.util.function.Function<T,T> f=this::identity; return f.apply(x); } }
`
- 四腿实际输入 jar / CLI5 candidate source hash：
  - `corretto8/debug` jar `76e9b4578b846b46fb0d503759b22f13d0a5d98d79c63b88a410a92e3e671030`；source MethodHandleUse=fb80f400fbab7e0180c6846cd314785d354662254fd78adba9fa57fdf980c5fd
  - `corretto8/nodebug` jar `41944535109c1af3d7ea94b6d75045f10e766b35e89956f09fc92247060de18e`；source MethodHandleUse=824a4363116dffd2001098a9dedb81c6a6c65971addce4b2e16ab51a0ed73610
  - `openjdk23/debug` jar `96bdcbc0b2b5962b800cdd64382466e4ee75d133e3d4d895887237df5288809a`；source MethodHandleUse=fb80f400fbab7e0180c6846cd314785d354662254fd78adba9fa57fdf980c5fd
  - `openjdk23/nodebug` jar `be97ae98de7e3052cab42696a060cdd3349ac44d30dc9c29414bbb9a6b762189`；source MethodHandleUse=824a4363116dffd2001098a9dedb81c6a6c65971addce4b2e16ab51a0ed73610
- CLI5 candidate 完整源 [MethodHandleUse.java](../candidate/candidate-v5/gc01-08-140/openjdk23/debug/MethodHandleUse/jarde/MethodHandleUse.java) SHA-256 `fb80f400fbab7e0180c6846cd314785d354662254fd78adba9fa57fdf980c5fd`；以下是实际保存源码摘录（行号为 candidate 文件行号）：
```java
1: // jarde: presentation of `MethodHandleUse` from the class file's own declaration and one recovery run per member.
2: // jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
3: // jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
4: public class MethodHandleUse<T> extends java.lang.Object {
5:     public MethodHandleUse() {
6:         // @method <init>()V
13:     // jarde: generic Signature projection refused for `identity(Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
14:     public java.lang.Object identity(java.lang.Object x) {
15:         // @method identity(Ljava/lang/Object;)Ljava/lang/Object;
21:     // jarde: generic Signature projection refused for `relay(Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
22:     public java.lang.Object relay(java.lang.Object x) {
23:         // @method relay(Ljava/lang/Object;)Ljava/lang/Object;
26:         java.util.function.Function f = this::identity;
```
- 代表物理输入 jar `96bdcbc0b2b5962b800cdd64382466e4ee75d133e3d4d895887237df5288809a` 的 javap tool `/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home/bin/javap` SHA-256 `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e`；保存的输入字节摘录：
  - `#40 = MethodHandle       5:#41          // REF_invokeVirtual MethodHandleUse.identity:(Ljava/lang/Object;)Ljava/lang/Object;`
  - `#43 = MethodHandle       6:#44          // REF_invokeStatic java/lang/invoke/LambdaMetafactory.metafactory:(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodHandle;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/CallSite;`
  - `1: invokespecial #1                  // Method java/lang/Object."<init>":()V`
  - `1: invokedynamic #7,  0              // InvokeDynamic #0:apply:(LMethodHandleUse;)Ljava/util/function/Function;`
  - `9: invokeinterface #11,  2           // InterfaceMethod java/util/function/Function.apply:(Ljava/lang/Object;)Ljava/lang/Object;`
  - `BootstrapMethods:`
  - `0: #43 REF_invokeStatic java/lang/invoke/LambdaMetafactory.metafactory:(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodHandle;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/CallSite;`
  - `#40 REF_invokeVirtual MethodHandleUse.identity:(Ljava/lang/Object;)Ljava/lang/Object;`
- CLI5 四腿 compile/Probe/behavior match 4/4；完整反射匹配 0/4；candidate behavior `behavior=method-handle-marker`。这只是实测，不代替具体判据。
- CLI4→CLI5 实际 refusal marker 差异：
  - CLI4 only: `// jarde: generic Signature projection refused for \`identity(Ljava/lang/Object;)Ljava/lang/Object;\`: same-class generic call dependency did not close over every incoming use`
  - CLI4 only: `// jarde: generic Signature projection refused for \`relay(Ljava/lang/Object;)Ljava/lang/Object;\`: same-class generic call dependency did not close over every incoming use`
  - CLI5 only: `// jarde: generic Signature projection refused for \`identity(Ljava/lang/Object;)Ljava/lang/Object;\`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload`
  - CLI5 only: `// jarde: generic Signature projection refused for \`relay(Ljava/lang/Object;)Ljava/lang/Object;\`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types`
- 证据未覆盖/边界：物理摘录可见 invokedynamic/bootstrap MethodHandle 指向 identity，但 relay 的直接调用是 Function.apply；未保存内部 call-graph/classifier trace，不能从 identity marker 推断 MethodHandle 已被正确排除在 direct-invoke 边界外。

### RawOwnReceiver（GC-05）

- 历史 root disposition：CLI3/CLI4 均为 `bounded_refusal`，`full_api_recovered=False`。既有支持范围：原raw alias被renderer折为this，但SSA实参receiver未获直接this/参数化receiver证明；整体保持raw而不从原源码或展示this猜T。
- 固定输入源码 SHA-256 `61a4b6d25d7525d6fab2685e1875a5f6944f21f6b644a766a4e6c74838912932`；原始源码实际文本：`public class RawOwnReceiver<T> { public T identity(T x) { return x; } @SuppressWarnings({"rawtypes","unchecked"}) public T relay(T x) { RawOwnReceiver raw=this; return (T)raw.identity(x); } }
`
- 四腿实际输入 jar / CLI5 candidate source hash：
  - `corretto8/debug` jar `86f184bd8b1d77ccf240c31aa4e4944e9f90ce001de7c5d543ce1c03361da943`；source RawOwnReceiver=9841e2b87c9c222434ec2f7bafc0e6438b13989a351e1b71d73baf9df378c4c8
  - `corretto8/nodebug` jar `bee7fcf9598de25917cefd4e8b915e35bb2b2e568dc41cfde2f8b939ab04e92f`；source RawOwnReceiver=6b6e312e35bab8fb84012b291deb55df266080add57f632ad937f3977079d208
  - `openjdk23/debug` jar `f4fd34adc6ab76cbce3d87a2beb62d6375ddfa045ebf19322cfd1d07130b7ce2`；source RawOwnReceiver=9841e2b87c9c222434ec2f7bafc0e6438b13989a351e1b71d73baf9df378c4c8
  - `openjdk23/nodebug` jar `4a8f260f57cabb8edf7a3ba6fab17f09e493b00a2cdd400128ad902b25b7e4fc`；source RawOwnReceiver=6b6e312e35bab8fb84012b291deb55df266080add57f632ad937f3977079d208
- CLI5 candidate 完整源 [RawOwnReceiver.java](../candidate/candidate-v5/gc01-08-140/openjdk23/debug/RawOwnReceiver/jarde/RawOwnReceiver.java) SHA-256 `9841e2b87c9c222434ec2f7bafc0e6438b13989a351e1b71d73baf9df378c4c8`；以下是实际保存源码摘录（行号为 candidate 文件行号）：
```java
1: // jarde: presentation of `RawOwnReceiver` from the class file's own declaration and one recovery run per member.
2: // jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
3: // jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
4: public class RawOwnReceiver<T> extends java.lang.Object {
5:     public RawOwnReceiver() {
6:         // @method <init>()V
13:     // jarde: generic Signature projection refused for `identity(Ljava/lang/Object;)Ljava/lang/Object;`: same-class generic call dependency did not close over every incoming use
14:     public java.lang.Object identity(java.lang.Object x) {
15:         // @method identity(Ljava/lang/Object;)Ljava/lang/Object;
21:     // jarde: generic Signature projection refused for `relay(Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
22:     public java.lang.Object relay(java.lang.Object x) {
23:         // @method relay(Ljava/lang/Object;)Ljava/lang/Object;
26:         return this.identity(x);
```
- 代表物理输入 jar `f4fd34adc6ab76cbce3d87a2beb62d6375ddfa045ebf19322cfd1d07130b7ce2` 的 javap tool `/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home/bin/javap` SHA-256 `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e`；保存的输入字节摘录：
  - `1: invokespecial #1                  // Method java/lang/Object."<init>":()V`
  - `4: invokevirtual #7                  // Method identity:(Ljava/lang/Object;)Ljava/lang/Object;`
- CLI5 四腿 compile/Probe/behavior match 4/4；完整反射匹配 0/4；candidate behavior `behavior=raw-receiver-marker`。这只是实测，不代替具体判据。
- CLI4→CLI5 实际 refusal marker 差异：
  - CLI4 only: `// jarde: generic Signature projection refused for \`relay(Ljava/lang/Object;)Ljava/lang/Object;\`: same-class generic call dependency did not close over every incoming use`
  - CLI5 only: `// jarde: generic Signature projection refused for \`relay(Ljava/lang/Object;)Ljava/lang/Object;\`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types`
- 证据未覆盖/边界：未保存 receiver local 的 AST/SSA 静态类型、cast/alias def-use 或绑定证明。RawOwnReceiver 原始 receiver 是 raw 局部，CLI5 展示源码将调用渲染为 `this.identity(...)`；ReboundOwnReceiver 原始变量声明静态类型一直是 `ReboundOwnReceiver<T>`，unchecked raw cast assignment 不改变其声明类型，而 CLI5 candidate 明确声明 raw local `ReboundOwnReceiver receiver`。只能支持“源码与 candidate 展示存在差异、内部证明未见”，不能把 candidate raw local 倒推为原源码 receiver 静态类型。

### ReboundOwnReceiver（GC-05）

- 历史 root disposition：CLI3/CLI4 均为 `bounded_refusal`，`full_api_recovered=False`。既有支持范围：原receiver静态类型仍为C<T>，unchecked raw赋值不改变其类型；候选的实际局部声明是raw C。Object alias、raw cast及重绑定传播未获证明，依实际发射保持拒绝。
- 固定输入源码 SHA-256 `d0010bfe7d04781b025af9b82b17eec54c71a22a890d883189da67ea4473ea81`；原始源码实际文本：`public class ReboundOwnReceiver<T> { public T identity(T x) { return x; } @SuppressWarnings({"rawtypes","unchecked"}) public T relay(T x) { ReboundOwnReceiver<T> receiver=this; Object alias=receiver; receiver=(ReboundOwnReceiver)alias; return (T)receiver.identity(x); } }
`
- 四腿实际输入 jar / CLI5 candidate source hash：
  - `corretto8/debug` jar `7e03c42cbe96cff0daa8d424e3fc237b2b0f1287f6697666aad5f8ec3d7710be`；source ReboundOwnReceiver=ff8bf4447ea94cc7832be42fff301f3a143eefb920fcf2d156e935e417c4e3e8
  - `corretto8/nodebug` jar `56d3743931a614c2240bfd164ab761d7ce65c781f2b02aca9cda59ec14f5f627`；source ReboundOwnReceiver=81e32ec2c2e5a186839326855521a0a4990aa021cc77f5ec7e93164328a19ce3
  - `openjdk23/debug` jar `ee70661aec76415d5ca4a3c296502ef4128f44b820d1ececa58ab8f7811e86bd`；source ReboundOwnReceiver=ff8bf4447ea94cc7832be42fff301f3a143eefb920fcf2d156e935e417c4e3e8
  - `openjdk23/nodebug` jar `463519ed62210f9854530be4a58868bf8c0cd6db0aae9a302cba29d5e33d9d14`；source ReboundOwnReceiver=81e32ec2c2e5a186839326855521a0a4990aa021cc77f5ec7e93164328a19ce3
- CLI5 candidate 完整源 [ReboundOwnReceiver.java](../candidate/candidate-v5/gc01-08-140/openjdk23/debug/ReboundOwnReceiver/jarde/ReboundOwnReceiver.java) SHA-256 `ff8bf4447ea94cc7832be42fff301f3a143eefb920fcf2d156e935e417c4e3e8`；以下是实际保存源码摘录（行号为 candidate 文件行号）：
```java
1: // jarde: presentation of `ReboundOwnReceiver` from the class file's own declaration and one recovery run per member.
2: // jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
3: // jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
4: public class ReboundOwnReceiver<T> extends java.lang.Object {
5:     public ReboundOwnReceiver() {
6:         // @method <init>()V
13:     // jarde: generic Signature projection refused for `identity(Ljava/lang/Object;)Ljava/lang/Object;`: same-class generic call dependency did not close over every incoming use
14:     public java.lang.Object identity(java.lang.Object x) {
15:         // @method identity(Ljava/lang/Object;)Ljava/lang/Object;
21:     // jarde: generic Signature projection refused for `relay(Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
22:     public java.lang.Object relay(java.lang.Object x) {
23:         // @method relay(Ljava/lang/Object;)Ljava/lang/Object;
26:         ReboundOwnReceiver receiver = this;
27:         ReboundOwnReceiver alias = receiver;
28:         receiver = (ReboundOwnReceiver) alias;
29:         return receiver.identity(x);
```
- 代表物理输入 jar `ee70661aec76415d5ca4a3c296502ef4128f44b820d1ececa58ab8f7811e86bd` 的 javap tool `/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home/bin/javap` SHA-256 `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e`；保存的输入字节摘录：
  - `1: invokespecial #1                  // Method java/lang/Object."<init>":()V`
  - `11: invokevirtual #9                  // Method identity:(Ljava/lang/Object;)Ljava/lang/Object;`
- CLI5 四腿 compile/Probe/behavior match 4/4；完整反射匹配 0/4；candidate behavior `behavior=raw-receiver-marker`。这只是实测，不代替具体判据。
- CLI4→CLI5 实际 refusal marker 差异：
  - CLI4 only: `// jarde: generic Signature projection refused for \`relay(Ljava/lang/Object;)Ljava/lang/Object;\`: same-class generic call dependency did not close over every incoming use`
  - CLI5 only: `// jarde: generic Signature projection refused for \`relay(Ljava/lang/Object;)Ljava/lang/Object;\`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types`
- 证据未覆盖/边界：未保存 receiver local 的 AST/SSA 静态类型、cast/alias def-use 或绑定证明。RawOwnReceiver 原始 receiver 是 raw 局部，CLI5 展示源码将调用渲染为 `this.identity(...)`；ReboundOwnReceiver 原始变量声明静态类型一直是 `ReboundOwnReceiver<T>`，unchecked raw cast assignment 不改变其声明类型，而 CLI5 candidate 明确声明 raw local `ReboundOwnReceiver receiver`。只能支持“源码与 candidate 展示存在差异、内部证明未见”，不能把 candidate raw local 倒推为原源码 receiver 静态类型。

### InheritedUnknown（GC-08）

- 历史 root disposition：CLI3/CLI4 均为 `bounded_refusal`，`full_api_recovered=False`。既有支持范围：符号owner写本类，但无本类物理get声明；父类ArrayList<T>替换与继承解析未获证明，class和method均保持可靠raw表示。
- 固定输入源码 SHA-256 `834b521f2fec6b6b750f4ef04ed8713c9cf29c4c90455fb027fb22b9ad7d48ac`；原始源码实际文本：`public class InheritedUnknown<T> extends java.util.ArrayList<T> { public T relay(int index) { return get(index); } }
`
- 四腿实际输入 jar / CLI5 candidate source hash：
  - `corretto8/debug` jar `772de6b969a14e4956ea1a52b39567856d0f08568b9de1d8c29c8751c0ffa4b9`；source InheritedUnknown=b1a19788afca0dba1724e77c2f258594ebe9fcc60c8d40b6ca5c47a55d24aefa
  - `corretto8/nodebug` jar `efd7b68c6d3004f5423c6d07a72088818bd2b93ebc5f973425718db9390f1784`；source InheritedUnknown=6ed6cce44fe8fe55b01f8bef2092c50a060f05499fca7bcba2e8c2376ed0f532
  - `openjdk23/debug` jar `4e6fff8b3902aee971cab85491b63d05411723be2ae9acb667a5fe5963ee4413`；source InheritedUnknown=b1a19788afca0dba1724e77c2f258594ebe9fcc60c8d40b6ca5c47a55d24aefa
  - `openjdk23/nodebug` jar `39aaef3ae858aba9b21c65dc8a4f0612eeb1574cfe05f9a864b60a5ef94b6249`；source InheritedUnknown=6ed6cce44fe8fe55b01f8bef2092c50a060f05499fca7bcba2e8c2376ed0f532
- CLI5 candidate 完整源 [InheritedUnknown.java](../candidate/candidate-v5/gc01-08-140/openjdk23/debug/InheritedUnknown/jarde/InheritedUnknown.java) SHA-256 `b1a19788afca0dba1724e77c2f258594ebe9fcc60c8d40b6ca5c47a55d24aefa`；以下是实际保存源码摘录（行号为 candidate 文件行号）：
```java
1: // jarde: presentation of `InheritedUnknown` from the class file's own declaration and one recovery run per member.
2: // jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
3: // jarde: class Signature `<T:Ljava/lang/Object;>Ljava/util/ArrayList<TT;>;` not projected
4: // jarde: class Signature projection refused: unsupported (class_generic_source_unproved): parameterized or nested parent needs a separate inherited-member proof
5: public class InheritedUnknown extends java.util.ArrayList {
6:     public InheritedUnknown() {
7:         // @method <init>()V
14:     // jarde: generic Signature projection refused for `relay(I)Ljava/lang/Object;`: unsupported (jvm_signature_scope_unproved): type variable `T` is not declared in the available Signature scope
15:     public java.lang.Object relay(int index) {
16:         // @method relay(I)Ljava/lang/Object;
19:         return this.get(index);
```
- 代表物理输入 jar `4e6fff8b3902aee971cab85491b63d05411723be2ae9acb667a5fe5963ee4413` 的 javap tool `/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home/bin/javap` SHA-256 `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e`；保存的输入字节摘录：
  - `1: invokespecial #1                  // Method java/util/ArrayList."<init>":()V`
  - `2: invokevirtual #7                  // Method get:(I)Ljava/lang/Object;`
- CLI5 四腿 compile/Probe/behavior match 4/4；完整反射匹配 0/4；candidate behavior `behavior=inherited-marker`。这只是实测，不代替具体判据。
- CLI4→CLI5 实际 refusal marker 差异：
  - CLI5 only: `// jarde: generic Signature projection refused for \`relay(I)Ljava/lang/Object;\`: unsupported (jvm_signature_scope_unproved): type variable \`T\` is not declared in the available Signature scope`
- 证据未覆盖/边界：CLI5 新增 relay 的 `jvm_signature_scope_unproved` marker，但没有保存内部 owner/member-resolution trace；所示 javap 摘录证明调用符号，不单独证明本类声明集合/继承解析。

### VarargsCall（GC-08）

- 历史 root disposition：CLI3/CLI4 均为 `bounded_refusal`，`full_api_recovered=False`。既有支持范围：ACC_VARARGS和调用前数组构造不能作固定参数direct formal代换；保留擦除varargs及调用行为，拒绝泛型头。
- 固定输入源码 SHA-256 `073e97bd1c4eaab7da3405a58d22561e5e1b86b3e20fac6c2551e9e08647d29b`；原始源码实际文本：`public class VarargsCall<T> { @SafeVarargs public final <U> U first(U... xs) { return xs[0]; } public T relay(T x) { return this.<T>first(x); } }
`
- 四腿实际输入 jar / CLI5 candidate source hash：
  - `corretto8/debug` jar `61f0a7c7e16c6aa610fdb2ee9c4d8bdf16c54d0a656ae67ce9e0f763bc70f8e1`；source VarargsCall=913c41baf1ae4866bf7da4c80ba0f6e01aa3f2b22ef24438acd28179b2c9634a
  - `corretto8/nodebug` jar `619f82efa329ca6a6ed59667922a1725a31f0b10881a2ad4aadc51b330c31885`；source VarargsCall=908a05872c2f7de6f2d04f5de7849083ed7c8dd4c32d059a7a8588bd85bfea84
  - `openjdk23/debug` jar `93b793a3ec3fc6afd845e5d2231b0bdc484bf16dcc8960a45933c830481314c7`；source VarargsCall=913c41baf1ae4866bf7da4c80ba0f6e01aa3f2b22ef24438acd28179b2c9634a
  - `openjdk23/nodebug` jar `b8a22da2c714908e2aaaa9a45516bcc8ac356879896ff5deecf305d792322615`；source VarargsCall=908a05872c2f7de6f2d04f5de7849083ed7c8dd4c32d059a7a8588bd85bfea84
- CLI5 candidate 完整源 [VarargsCall.java](../candidate/candidate-v5/gc01-08-140/openjdk23/debug/VarargsCall/jarde/VarargsCall.java) SHA-256 `913c41baf1ae4866bf7da4c80ba0f6e01aa3f2b22ef24438acd28179b2c9634a`；以下是实际保存源码摘录（行号为 candidate 文件行号）：
```java
1: // jarde: presentation of `VarargsCall` from the class file's own declaration and one recovery run per member.
2: // jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
3: // jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
4: public class VarargsCall<T> extends java.lang.Object {
5:     public VarargsCall() {
6:         // @method <init>()V
13:     // jarde: generic Signature projection refused for `first([Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
15:     public final java.lang.Object first(java.lang.Object... xs) {
16:         // @method first([Ljava/lang/Object;)Ljava/lang/Object;
22:     // jarde: generic Signature projection refused for `relay(Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
23:     public java.lang.Object relay(java.lang.Object x) {
24:         // @method relay(Ljava/lang/Object;)Ljava/lang/Object;
27:         return this.first(x);
```
- 代表物理输入 jar `93b793a3ec3fc6afd845e5d2231b0bdc484bf16dcc8960a45933c830481314c7` 的 javap tool `/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home/bin/javap` SHA-256 `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e`；保存的输入字节摘录：
  - `1: invokespecial #1                  // Method java/lang/Object."<init>":()V`
  - `flags: (0x0091) ACC_PUBLIC, ACC_FINAL, ACC_VARARGS`
  - `9: invokevirtual #7                  // Method first:([Ljava/lang/Object;)Ljava/lang/Object;`
- CLI5 四腿 compile/Probe/behavior match 4/4；完整反射匹配 0/4；candidate behavior `behavior=varargs-marker`。这只是实测，不代替具体判据。
- CLI4→CLI5 实际 refusal marker 差异：
  - CLI4 only: `// jarde: generic Signature projection refused for \`first([Ljava/lang/Object;)Ljava/lang/Object;\`: same-class generic call dependency did not close over every incoming use`
  - CLI4 only: `// jarde: generic Signature projection refused for \`relay(Ljava/lang/Object;)Ljava/lang/Object;\`: same-class generic call dependency did not close over every incoming use`
  - CLI5 only: `// jarde: generic Signature projection refused for \`first([Ljava/lang/Object;)Ljava/lang/Object;\`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload`
  - CLI5 only: `// jarde: generic Signature projection refused for \`relay(Ljava/lang/Object;)Ljava/lang/Object;\`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types`
- 证据未覆盖/边界：物理 ACC_VARARGS、数组 descriptor 与 invoke 指令有记录；未保存 varargs applicability/固定参数过滤 trace。展示源码省去 `<T>`，marker 文本不能证明该 invocation 被正确分类。

### SameErasureBinder（GC-04）

- 历史 root disposition：CLI3/CLI4 均为 `bounded_refusal`，`full_api_recovered=False`。既有支持范围：class T与callee U虽同Number擦除，U还需Runnable且本调用为null无唯一方法变量源；保留sink raw和独立class T，不计U恢复。
- 固定输入源码 SHA-256 `039e59c313e43331e9c658b84caeb056be5e435ac5bbae951cf0c1de5dbacd4d`；原始源码实际文本：`public class SameErasureBinder<T extends Number> { public int calls; public <U extends Number & Runnable> U sink(U x) { calls++; return x; } public T independent(T x) { return x; } public void useNull() { sink(null); } }
`
- 四腿实际输入 jar / CLI5 candidate source hash：
  - `corretto8/debug` jar `d2a8ba5de0f4b7d2da306df5f7b11932014b1616c734ef6488acc9ac65cf31ad`；source SameErasureBinder=55c22e2b994c2d28a6df7e28e1b7602f772ec05183ebfc78f0ed6d4c872eccea
  - `corretto8/nodebug` jar `919b9c4fed9fd5e73b5edb7f2ecea834a67c2964568d910c6f040267f6260ab8`；source SameErasureBinder=ffa161ba83deae05500641df11e9532a71a8757875f4efd299bb86057f838c81
  - `openjdk23/debug` jar `6a4836c9be87000f4f6959415ce47b452ced7e25c64c79ca64de66ff3c572478`；source SameErasureBinder=55c22e2b994c2d28a6df7e28e1b7602f772ec05183ebfc78f0ed6d4c872eccea
  - `openjdk23/nodebug` jar `571cfc3f8df4f5d139159984f7a9db618e6ec86ee01f3df9ff3a8a135d16edd1`；source SameErasureBinder=ffa161ba83deae05500641df11e9532a71a8757875f4efd299bb86057f838c81
- CLI5 candidate 完整源 [SameErasureBinder.java](../candidate/candidate-v5/gc01-08-140/openjdk23/debug/SameErasureBinder/jarde/SameErasureBinder.java) SHA-256 `55c22e2b994c2d28a6df7e28e1b7602f772ec05183ebfc78f0ed6d4c872eccea`；以下是实际保存源码摘录（行号为 candidate 文件行号）：
```java
1: // jarde: presentation of `SameErasureBinder` from the class file's own declaration and one recovery run per member.
2: // jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
3: // jarde: class Signature `<T:Ljava/lang/Number;>Ljava/lang/Object;` projected after physical parent erasure proof
4: public class SameErasureBinder<T extends java.lang.Number> extends java.lang.Object {
7:     public SameErasureBinder() {
8:         // @method <init>()V
15:     // jarde: generic Signature projection refused for `sink(Ljava/lang/Number;)Ljava/lang/Number;`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return
16:     public java.lang.Number sink(java.lang.Number x) {
17:         // @method sink(Ljava/lang/Number;)Ljava/lang/Number;
24:     public T independent(T x) {
26:         // @method independent(Ljava/lang/Number;)Ljava/lang/Number;
32:     public void useNull() {
33:         // @method useNull()V
36:         this.sink((java.lang.Number) null);
```
- 代表物理输入 jar `6a4836c9be87000f4f6959415ce47b452ced7e25c64c79ca64de66ff3c572478` 的 javap tool `/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home/bin/javap` SHA-256 `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e`；保存的输入字节摘录：
  - `1: invokespecial #1                  // Method java/lang/Object."<init>":()V`
  - `2: invokevirtual #13                 // Method sink:(Ljava/lang/Number;)Ljava/lang/Number;`
- CLI5 四腿 compile/Probe/behavior match 4/4；完整反射匹配 0/4；candidate behavior `behavior=null-call-count=1`。这只是实测，不代替具体判据。
- CLI4→CLI5 实际 refusal marker 差异：
  - CLI4 only: `// jarde: generic Signature projection refused for \`sink(Ljava/lang/Number;)Ljava/lang/Number;\`: same-class generic call dependency did not close over every incoming use`
  - CLI5 only: `// jarde: generic Signature projection refused for \`sink(Ljava/lang/Number;)Ljava/lang/Number;\`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return`
- 证据未覆盖/边界：候选保留 `sink(Number)` raw 与 `independent(T)` generic；输入源、invoke 和 Probe 均保存，但未保存 binder identity/formal-owner 的 AST proof trace。固定拒绝谓词 4/4 pass 不是 API recovery。

### MultiUseResult（GC-08）

- 历史 root disposition：CLI3/CLI4 均为 `bounded_refusal`，`full_api_recovered=False`。既有支持范围：call结果保存为局部并用于observe和return，多use/局部传播在范围外；完整incoming不闭合时identity和relay共同raw，实际observe与返回效果通过。
- 固定输入源码 SHA-256 `218fe6e2973099667db77c1d9936f148a5001ac6501671d4989f791dc0a0ef84`；原始源码实际文本：`public class MultiUseResult<T> { public Object observed; public T identity(T x) { return x; } public void observe(Object x) { observed=x; } public T relay(T x) { T result=identity(x); observe(result); return result; } }
`
- 四腿实际输入 jar / CLI5 candidate source hash：
  - `corretto8/debug` jar `26c42ab150ae61d478583cc9bb1bd4076dde60efa50202ff56491f95d78531f8`；source MultiUseResult=5b75c5ad433f9c590fd0dc3bbbf400d1e8c5565f134c952b960ec2362efb1ddd
  - `corretto8/nodebug` jar `8d5b40d975a71acc2abe5d9d6a33f6b845adfc20dfa209bfc07fc045806e1fd7`；source MultiUseResult=86c0e55a82f12945a5db96dcfeff0f51e36b59023c8ed152424458edf59296b2
  - `openjdk23/debug` jar `08749c04b3dcce9459e131384ab38e40aeb1da1514aaf4c9f8a4eb0c99ef0cb3`；source MultiUseResult=5b75c5ad433f9c590fd0dc3bbbf400d1e8c5565f134c952b960ec2362efb1ddd
  - `openjdk23/nodebug` jar `177ae21dbe67d969dae604f092c0e6c6acbab5a6585cc04f62cf6f4fd2899361`；source MultiUseResult=86c0e55a82f12945a5db96dcfeff0f51e36b59023c8ed152424458edf59296b2
- CLI5 candidate 完整源 [MultiUseResult.java](../candidate/candidate-v5/gc01-08-140/openjdk23/debug/MultiUseResult/jarde/MultiUseResult.java) SHA-256 `5b75c5ad433f9c590fd0dc3bbbf400d1e8c5565f134c952b960ec2362efb1ddd`；以下是实际保存源码摘录（行号为 candidate 文件行号）：
```java
1: // jarde: presentation of `MultiUseResult` from the class file's own declaration and one recovery run per member.
2: // jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
3: // jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
4: public class MultiUseResult<T> extends java.lang.Object {
7:     public MultiUseResult() {
8:         // @method <init>()V
15:     // jarde: generic Signature projection refused for `identity(Ljava/lang/Object;)Ljava/lang/Object;`: same-class generic call dependency did not close over every incoming use
16:     public java.lang.Object identity(java.lang.Object x) {
17:         // @method identity(Ljava/lang/Object;)Ljava/lang/Object;
23:     public void observe(java.lang.Object x) {
24:         // @method observe(Ljava/lang/Object;)V
31:     // jarde: generic Signature projection refused for `relay(Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
32:     public java.lang.Object relay(java.lang.Object x) {
33:         // @method relay(Ljava/lang/Object;)Ljava/lang/Object;
36:         java.lang.Object result = this.identity(x);
37:         this.observe(result);
```
- 代表物理输入 jar `08749c04b3dcce9459e131384ab38e40aeb1da1514aaf4c9f8a4eb0c99ef0cb3` 的 javap tool `/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home/bin/javap` SHA-256 `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e`；保存的输入字节摘录：
  - `1: invokespecial #1                  // Method java/lang/Object."<init>":()V`
  - `2: invokevirtual #13                 // Method identity:(Ljava/lang/Object;)Ljava/lang/Object;`
  - `8: invokevirtual #17                 // Method observe:(Ljava/lang/Object;)V`
- CLI5 四腿 compile/Probe/behavior match 4/4；完整反射匹配 0/4；candidate behavior `behavior=return+observer-marker`。这只是实测，不代替具体判据。
- CLI4→CLI5 实际 refusal marker 差异：
  - CLI4 only: `// jarde: generic Signature projection refused for \`relay(Ljava/lang/Object;)Ljava/lang/Object;\`: same-class generic call dependency did not close over every incoming use`
  - CLI5 only: `// jarde: generic Signature projection refused for \`relay(Ljava/lang/Object;)Ljava/lang/Object;\`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types`
- 证据未覆盖/边界：candidate 源可见 Object result local 同时传给 observe 并 return，物理输入有 identity/observe invoke；未保存 consumer-edge/atomic-erased-fallback 内部 trace。4/4 是固定拒绝谓词结果，不是 API recovery。

### BridgeUnknown（GC-08）

- 历史 root disposition：CLI3/CLI4 均为 `positive_control`，`full_api_recovered=True`。既有支持范围：既有pure桥接证明成立，javac重编补桥，保留真实String checkcast；四腿完整API匹配，作为正控制。
- 固定输入源码 SHA-256 `ed6f25727a6ef7554b35e416ce6b58e25261c5da39256934d9e202c45dabb41d`；原始源码实际文本：`class BridgeBase<T> { public T apply(T x) { return x; } }
public class BridgeUnknown extends BridgeBase<String> { @Override public String apply(String x) { return x; } public Object relay(Object x) { return apply((String)x); } }
`
- 四腿实际输入 jar / CLI5 candidate source hash：
  - `corretto8/debug` jar `3823cd1f8756394e9e1f437728da89e145e9efd825f2334c3985e943a3df9f72`；source BridgeBase=bb3261d2ac87c52dfd9500a7ffab703c57a98610f25634e5b50293f054f422ce, BridgeUnknown=b2ba5183c053b297e145505543afa6fe829062fec257fb5229f49e10873ab39f
  - `corretto8/nodebug` jar `66a13df08756d49bc9a6078f7f673b89b03afa3df404c1363b5bd378cf38b0d4`；source BridgeBase=4de6b267b4ebcc85f7717634f40ad35a0023073e9bfda5a9de1ba60a5bfcb108, BridgeUnknown=6ef39b72c339d5499ff91ae5e8083d568edd6fbe17e796f1035fac03bf6f6742
  - `openjdk23/debug` jar `350f04a0901b2988b9d46db166fba891ba1694e9f7a25a808f9c3fa0e7423ca0`；source BridgeBase=bb3261d2ac87c52dfd9500a7ffab703c57a98610f25634e5b50293f054f422ce, BridgeUnknown=b2ba5183c053b297e145505543afa6fe829062fec257fb5229f49e10873ab39f
  - `openjdk23/nodebug` jar `69ea0c814fc5c13fe11f141e5c742016e54f43d226028a809a32cf0f402a4762`；source BridgeBase=4de6b267b4ebcc85f7717634f40ad35a0023073e9bfda5a9de1ba60a5bfcb108, BridgeUnknown=6ef39b72c339d5499ff91ae5e8083d568edd6fbe17e796f1035fac03bf6f6742
- CLI5 candidate 完整源 [BridgeUnknown.java](../candidate/candidate-v5/gc01-08-140/openjdk23/debug/BridgeUnknown/jarde/BridgeUnknown.java) SHA-256 `b2ba5183c053b297e145505543afa6fe829062fec257fb5229f49e10873ab39f`；以下是实际保存源码摘录（行号为 candidate 文件行号）：
```java
1: // jarde: presentation of `BridgeUnknown` from the class file's own declaration and one recovery run per member.
2: // jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
3: // jarde: class Signature `LBridgeBase<Ljava/lang/String;>;` projected after physical parent erasure proof
4: public class BridgeUnknown extends BridgeBase<java.lang.String> {
5:     public BridgeUnknown() {
6:         // @method <init>()V
13:     public java.lang.String apply(java.lang.String x) {
14:         // @method apply(Ljava/lang/String;)Ljava/lang/String;
20:     public java.lang.Object relay(java.lang.Object x) {
21:         // @method relay(Ljava/lang/Object;)Ljava/lang/Object;
24:         return this.apply((java.lang.String) x);
```
- 代表物理输入 jar `350f04a0901b2988b9d46db166fba891ba1694e9f7a25a808f9c3fa0e7423ca0` 的 javap tool `/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home/bin/javap` SHA-256 `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e`；保存的输入字节摘录：
  - `1: invokespecial #1                  // Method BridgeBase."<init>":()V`
  - `5: invokevirtual #9                  // Method apply:(Ljava/lang/String;)Ljava/lang/String;`
- CLI5 四腿 compile/Probe/behavior match 4/4；完整反射匹配 4/4；candidate behavior `behavior=bridge-marker`。这只是实测，不代替具体判据。
- CLI4→CLI5 候选完整源码字节一致；class/method 头、调用表达式和 marker 无变。
- 证据未覆盖/边界：CLI5 源码与 CLI4 字节一致；candidate 继承 BridgeBase<String>，relay 展示 String cast，input invoke 是 apply(String)，反射 4/4 match。内部 bridge ownership proof trace 未保存；保留为正控制，不并入拒绝。

### IncompleteSite（GC-08）

- 历史 root disposition：CLI3/CLI4 均为 `positive_control`，`full_api_recovered=True`。既有支持范围：原fixture有合法条件单invoke，四腿两分支和API都匹配，作为正控制；缺项清单由独立故障测试验证，不改该输入凑负例。
- 固定输入源码 SHA-256 `3d4c9f64d35222cb1e6591e24f64bdee3178f1d0370ca09bd9356b51d4d55e0a`；原始源码实际文本：`public class IncompleteSite<T> { public T identity(T x) { return x; } public T relay(T x, boolean use) { if (use) return identity(x); return x; } }
`
- 四腿实际输入 jar / CLI5 candidate source hash：
  - `corretto8/debug` jar `8caf43e2252d114b114d51193e5cd3153fac096f13a48889fc4e88c4f8a1a27b`；source IncompleteSite=56fc806f2c0e6b908118f83d0b6ed0e4bd286205a7f9fcd311d5b25f1d2ef02f
  - `corretto8/nodebug` jar `c659525e0fa7cb64236e08ea4c643bf182881786617c4fea5ab820fb9405d55e`；source IncompleteSite=c22257bfc7fa7dddd45bd7a310887b51506168edcd590403527efe7e205952b1
  - `openjdk23/debug` jar `26c16dc2130a6a63193a0535016e519b2f048733d9b68cadef7950ba50bdb94f`；source IncompleteSite=56fc806f2c0e6b908118f83d0b6ed0e4bd286205a7f9fcd311d5b25f1d2ef02f
  - `openjdk23/nodebug` jar `52dc3a3143e5b2e20632bcfb592cfcd8a59055c664bc5b6276c5c763f864c7fd`；source IncompleteSite=c22257bfc7fa7dddd45bd7a310887b51506168edcd590403527efe7e205952b1
- CLI5 candidate 完整源 [IncompleteSite.java](../candidate/candidate-v5/gc01-08-140/openjdk23/debug/IncompleteSite/jarde/IncompleteSite.java) SHA-256 `56fc806f2c0e6b908118f83d0b6ed0e4bd286205a7f9fcd311d5b25f1d2ef02f`；以下是实际保存源码摘录（行号为 candidate 文件行号）：
```java
1: // jarde: presentation of `IncompleteSite` from the class file's own declaration and one recovery run per member.
2: // jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
3: // jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
4: public class IncompleteSite<T> extends java.lang.Object {
5:     public IncompleteSite() {
6:         // @method <init>()V
13:     public T identity(T x) {
15:         // @method identity(Ljava/lang/Object;)Ljava/lang/Object;
21:     public T relay(T x, boolean use) {
23:         // @method relay(Ljava/lang/Object;Z)Ljava/lang/Object;
27:             return this.identity(x);
```
- 代表物理输入 jar `26c16dc2130a6a63193a0535016e519b2f048733d9b68cadef7950ba50bdb94f` 的 javap tool `/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home/bin/javap` SHA-256 `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e`；保存的输入字节摘录：
  - `1: invokespecial #1                  // Method java/lang/Object."<init>":()V`
  - `6: invokevirtual #7                  // Method identity:(Ljava/lang/Object;)Ljava/lang/Object;`
- CLI5 四腿 compile/Probe/behavior match 4/4；完整反射匹配 4/4；candidate behavior `behavior=both-conditional-paths`。这只是实测，不代替具体判据。
- CLI4→CLI5 候选完整源码字节一致；class/method 头、调用表达式和 marker 无变。
- 证据未覆盖/边界：CLI5 源码与 CLI4 字节一致；保存的源显示合法条件单调用与两分支，input 有 identity invoke，反射 4/4 match。该 fixture 不证明其他缺项 census 安全，仍保留为正控制。

## 复核结论边界

1. CLI3/CLI4 root 的 9 个历史 `bounded_refusal`（七项需复核控制 + SameErasureBinder + MultiUseResult）和 2 个 `positive_control`，及其支持理由，在两个 disposition 文件内逐项相同。本页不把历史裁定复制成 CLI5 新裁定。
2. CLI5 的 44 行输入源码、jar、Probe 均沿用同一冻结题面；candidate 方面 36 行（七项需复核 + 两拒绝正控）源码 hash 改变，8 行（BridgeUnknown/IncompleteSite）字节不变。CLI5 类/方法声明头没有变化；变化集中在拒绝 marker 原因/补充。CLI5 facts 中 44 条 candidate Probe stdout/behavior 与 CLI4 一致。
3. invoke/bootstrap/ACC_VARARGS 是输入字节证据；展示源码是 CLI5 实际发射结果；两者之间的 AST receiver、owner、binder、consumer、varargs proof chain 没有作为逐调用 trace 冻结。若 root 要把拒绝边界裁为最终通过，应额外核对实现内证据；编译/行为通过、marker 文本、或单看展示 receiver 均不能替代。
4. CLI5 本轮 measured matrix remains `needs-review`。本文件只写新审计辅助证据，不改验收判据、v3/v4 disposition、生产、测试、Cargo 或 Git；未运行 target。

## 输入字节的直接 javap 静态复读（本次只读，不运行 class）

为避免把旧 disposition 当作物理事实，本页生成后又直接对 CLI5 run 保存的 44 个 input jar 分别执行 `JDK_HOME/bin/javap -v -p -classpath <that-input.jar> <class>`；没有运行 fixture、Probe 或候选程序。44/44 命令 exit=0，jar 与 CLI5 facts 中 SHA 一致，javap binary SHA 与 CLI5 matrix manifest 对应 JDK identity 一致。下表记录每个 command 的 jar、tool、captured stdout SHA-256；raw stdout 未额外保存，代表腿的实际指令/bootstrap 摘录直接列在表后。

| Case | Leg/mode | Input jar SHA-256 | javap tool SHA-256 | Captured javap stdout SHA-256 | Exit |
| --- | --- | --- | --- | --- | ---: |
| UnknownIncoming | corretto8/debug | `72327f51b077244ba5fe2896a7c8897acfd7e326ece2bc1c0c1fa1270bea519b` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `e060888c5ff09c9d2fd920959e02f301e0c0265498c73dab8d6c40ef5f5f02d5` | 0 |
| UnknownIncoming | corretto8/nodebug | `ea3309ef41feae4efe5091976a54899f25d1627ae4e572b7f5635669cb691577` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `063047bf4c833eda1fd055e0183b1f9656c4a58e1a1399575391d8446ed48101` | 0 |
| UnknownIncoming | openjdk23/debug | `dbe1f18fa7dcec4135024bfbe68d3c5a61d358a45ea69f63e2d0b20ecf314447` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `957e72de4d6c62adde8b4e301fc360edc734b80ae725bca9feb3a4f416e31ebf` | 0 |
| UnknownIncoming | openjdk23/nodebug | `6b1d92d63160c91655e838f87ba7ceccc5066dbba1bcaa01150fd6b4576cc2c3` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `23ba94b3c96c1fa3cd96669738d1b5f8479881c51921acae5ad8114506b56700` | 0 |
| CycleRelay | corretto8/debug | `0a19c1468a138d4686dd223683ead772576d5527ff27b176b61aab00442eb4f2` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `0aeceabc869cb584914068d17d9ba1480794171f5fb70b02878cab37865e1657` | 0 |
| CycleRelay | corretto8/nodebug | `93811aca27631b11d07b2be9f08e81612240e215f697b263633d2116e63cf808` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `1b3b6774e9eb1f29592ed76e99f6df3de08e1a7059b2b14d9b03f8f064ff4f58` | 0 |
| CycleRelay | openjdk23/debug | `1436b39ebfabb2d3b47e1a42a0b0152b803c4d640f467afbad630b4f8c748b6e` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `3667bedd23b8d4b3cdcef6805b614cedb719bd6b095fd2ad7b74ba6d32f3c6ca` | 0 |
| CycleRelay | openjdk23/nodebug | `4724274f1301d83a75972e00b6071a02ab5b3f2d89f7a2fca84fde33273c369f` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `c71014dd37509d51d1fd106150aaf1dbb2149318177f140c6802e338bc8cb647` | 0 |
| MethodHandleUse | corretto8/debug | `76e9b4578b846b46fb0d503759b22f13d0a5d98d79c63b88a410a92e3e671030` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `97933d58d4500690608e68331811a2c69b7953ccf7b6a879920c99a415372cde` | 0 |
| MethodHandleUse | corretto8/nodebug | `41944535109c1af3d7ea94b6d75045f10e766b35e89956f09fc92247060de18e` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `d8b246b7fe172e0fae93711251526ea64c06ce568948aaec8b183f39d430123d` | 0 |
| MethodHandleUse | openjdk23/debug | `96bdcbc0b2b5962b800cdd64382466e4ee75d133e3d4d895887237df5288809a` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `da1ac49ae8ce089987ce82225c1310b74bd410f17f3b027ec5c98ed7939e4979` | 0 |
| MethodHandleUse | openjdk23/nodebug | `be97ae98de7e3052cab42696a060cdd3349ac44d30dc9c29414bbb9a6b762189` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `3d3cd1665a4ef73ce0281f62ebb85d026349121b8d14590e604d4ea82cc67fbd` | 0 |
| RawOwnReceiver | corretto8/debug | `86f184bd8b1d77ccf240c31aa4e4944e9f90ce001de7c5d543ce1c03361da943` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `e83c7903a303ec610d3ef30694908cf93a750a256a97801ceaa62abf7567d45c` | 0 |
| RawOwnReceiver | corretto8/nodebug | `bee7fcf9598de25917cefd4e8b915e35bb2b2e568dc41cfde2f8b939ab04e92f` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `99b0302b0a991633518835f27b22db4e78ae584913171b17369a2cd2cc1e56f5` | 0 |
| RawOwnReceiver | openjdk23/debug | `f4fd34adc6ab76cbce3d87a2beb62d6375ddfa045ebf19322cfd1d07130b7ce2` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `5d2b085d17491cad25df0542c81529da35fdb4edffa2fbb02348d51e9d51a83c` | 0 |
| RawOwnReceiver | openjdk23/nodebug | `4a8f260f57cabb8edf7a3ba6fab17f09e493b00a2cdd400128ad902b25b7e4fc` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `9cfb47fa0613a76dedb2f2ff12acf3b59a77e83e010b83f17b71f5ac206a5844` | 0 |
| ReboundOwnReceiver | corretto8/debug | `7e03c42cbe96cff0daa8d424e3fc237b2b0f1287f6697666aad5f8ec3d7710be` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `04d86482c55e1e137b46853c88e680c1df8554ef2ec42a2c1bdd55feedcb0360` | 0 |
| ReboundOwnReceiver | corretto8/nodebug | `56d3743931a614c2240bfd164ab761d7ce65c781f2b02aca9cda59ec14f5f627` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `8e6ad3770b4bc71e51797875174fe353470a0f26d811088d6f5a8c7b268f89c5` | 0 |
| ReboundOwnReceiver | openjdk23/debug | `ee70661aec76415d5ca4a3c296502ef4128f44b820d1ececa58ab8f7811e86bd` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `83d70cd5fec70cae6f1a4c1eb1d662d944b721c3551d8df051d738908cd2450c` | 0 |
| ReboundOwnReceiver | openjdk23/nodebug | `463519ed62210f9854530be4a58868bf8c0cd6db0aae9a302cba29d5e33d9d14` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `c1193fe813b3ff05f1c67d5bf63829d38b73685b4db34032136c7792f65325ce` | 0 |
| InheritedUnknown | corretto8/debug | `772de6b969a14e4956ea1a52b39567856d0f08568b9de1d8c29c8751c0ffa4b9` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `9ff67543e69a9d684dddb791358fcff4bfdfbf50f7791875ceba42ab2b17b84b` | 0 |
| InheritedUnknown | corretto8/nodebug | `efd7b68c6d3004f5423c6d07a72088818bd2b93ebc5f973425718db9390f1784` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `dfa0cb731f988982f7a6369a86916369c321e94ef2e6d4cd9895a7390cca3fa0` | 0 |
| InheritedUnknown | openjdk23/debug | `4e6fff8b3902aee971cab85491b63d05411723be2ae9acb667a5fe5963ee4413` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `abfec63a21f985030f16ac8a72e8c0956faaa1157999abee64d7bda0dbc3cc6b` | 0 |
| InheritedUnknown | openjdk23/nodebug | `39aaef3ae858aba9b21c65dc8a4f0612eeb1574cfe05f9a864b60a5ef94b6249` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `a1063ad02301c738f224bd6a01307a7feda769744e4c4f287ca75b67a3f379ec` | 0 |
| VarargsCall | corretto8/debug | `61f0a7c7e16c6aa610fdb2ee9c4d8bdf16c54d0a656ae67ce9e0f763bc70f8e1` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `8d9277aa31db82d6c12182b08af5e0aa27641e5291b86a634424e43a08469f93` | 0 |
| VarargsCall | corretto8/nodebug | `619f82efa329ca6a6ed59667922a1725a31f0b10881a2ad4aadc51b330c31885` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `56634bbcefe22f3bffdf023a6923508ef396fa939a418e7516023fad07d2a92e` | 0 |
| VarargsCall | openjdk23/debug | `93b793a3ec3fc6afd845e5d2231b0bdc484bf16dcc8960a45933c830481314c7` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `79c66cba7cb7f2f850debbe33e411b1c25b4366664c8756e7bdb00b488fb21f2` | 0 |
| VarargsCall | openjdk23/nodebug | `b8a22da2c714908e2aaaa9a45516bcc8ac356879896ff5deecf305d792322615` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `89f8d0cbd90d8d0520e715ec0bfa1b13d2f94a2e4feeb45e17ea23c47ce2dac2` | 0 |
| SameErasureBinder | corretto8/debug | `d2a8ba5de0f4b7d2da306df5f7b11932014b1616c734ef6488acc9ac65cf31ad` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `ed9b2fe5bbee0e0684f92ec89ba36f345c4f96a288fb23723b5df1b2bf45f0ea` | 0 |
| SameErasureBinder | corretto8/nodebug | `919b9c4fed9fd5e73b5edb7f2ecea834a67c2964568d910c6f040267f6260ab8` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `d30741b48d4baa2cc8a75d46b34da910ebf0ac690888b96fc06ab7a1b453ed8c` | 0 |
| SameErasureBinder | openjdk23/debug | `6a4836c9be87000f4f6959415ce47b452ced7e25c64c79ca64de66ff3c572478` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `eaffff761da2073f58dca6301be32290cc920f9329980f4368adcd750251ffa9` | 0 |
| SameErasureBinder | openjdk23/nodebug | `571cfc3f8df4f5d139159984f7a9db618e6ec86ee01f3df9ff3a8a135d16edd1` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `a896a399f4d36779136a3eab10372ed4d6549a407e4258f7a58075640fea4e5d` | 0 |
| MultiUseResult | corretto8/debug | `26c42ab150ae61d478583cc9bb1bd4076dde60efa50202ff56491f95d78531f8` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `2d8e73cdaa24098472547e07f9a705da15bc120d7d7028c04010217f9e92d42e` | 0 |
| MultiUseResult | corretto8/nodebug | `8d5b40d975a71acc2abe5d9d6a33f6b845adfc20dfa209bfc07fc045806e1fd7` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `38c2de8eaebe6e38ed9ca567836adee3670f1dc16a6a369e5b3b26cf5383c2b1` | 0 |
| MultiUseResult | openjdk23/debug | `08749c04b3dcce9459e131384ab38e40aeb1da1514aaf4c9f8a4eb0c99ef0cb3` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `82992c3e49a62391994f0eccd6f346c7ff20610a643d7b96b591eb4d14bdab57` | 0 |
| MultiUseResult | openjdk23/nodebug | `177ae21dbe67d969dae604f092c0e6c6acbab5a6585cc04f62cf6f4fd2899361` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `610e30d3f075cdbd073f56a35014a86a5b925e13a618350de8cf54bf03942af1` | 0 |
| BridgeUnknown | corretto8/debug | `3823cd1f8756394e9e1f437728da89e145e9efd825f2334c3985e943a3df9f72` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `10d7695097e78d04f1f5567e9bb3c6381ac6df23741b5e1bc8ca0cb591b69a3c` | 0 |
| BridgeUnknown | corretto8/nodebug | `66a13df08756d49bc9a6078f7f673b89b03afa3df404c1363b5bd378cf38b0d4` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `ef03c743c956a5d2130fa4630b89109716e0b0b0fd4de42cdccade077c01933a` | 0 |
| BridgeUnknown | openjdk23/debug | `350f04a0901b2988b9d46db166fba891ba1694e9f7a25a808f9c3fa0e7423ca0` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `48654eed1fe51ef55fb563cf2288d62d03f5704ef9d8d422355914460d65f1be` | 0 |
| BridgeUnknown | openjdk23/nodebug | `69ea0c814fc5c13fe11f141e5c742016e54f43d226028a809a32cf0f402a4762` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `8e091ad0ed2ad50305e5a926715b1a29abc81faf8cc4429c3c3f1b6a264e050d` | 0 |
| IncompleteSite | corretto8/debug | `8caf43e2252d114b114d51193e5cd3153fac096f13a48889fc4e88c4f8a1a27b` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `565983f09094a88eea2a76960140cffd3d478f1c4cef21f65006993dbc499081` | 0 |
| IncompleteSite | corretto8/nodebug | `c659525e0fa7cb64236e08ea4c643bf182881786617c4fea5ab820fb9405d55e` | `fe06c052822fef0e9d8398650bd88bf9cf129404d59cfc600056b0e4ca0e1ac1` | `07734f7cc4d1b52e7da4ac8ac345f1a7bbd27367cac5ecdf1c48ef5c588cc4d2` | 0 |
| IncompleteSite | openjdk23/debug | `26c16dc2130a6a63193a0535016e519b2f048733d9b68cadef7950ba50bdb94f` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `e9ce5e3613007dbf7a26de209b43a80a59bddfe17e6b1210ee497d78a4ec9c89` | 0 |
| IncompleteSite | openjdk23/nodebug | `52dc3a3143e5b2e20632bcfb592cfcd8a59055c664bc5b6276c5c763f864c7fd` | `f8e5fca8a790cc2f82cf2fa6a5a8d34dad2550a79ba553260df249d06602198e` | `46e46027644b4de3d2a037396219ccdea5e9da1cf9e878bcb91e495565afd828` | 0 |

代表物理行（openjdk23/debug，逐字摘自上述新 javap stdout）：

**UnknownIncoming**（jar SHA `dbe1f18fa7dcec4135024bfbe68d3c5a61d358a45ea69f63e2d0b20ecf314447`）：
- `2: invokevirtual #7                  // Method identity:(Ljava/lang/Object;)Ljava/lang/Object;`
**CycleRelay**（jar SHA `1436b39ebfabb2d3b47e1a42a0b0152b803c4d640f467afbad630b4f8c748b6e`）：
- `7: invokevirtual #7                  // Method right:(Ljava/lang/Object;Z)Ljava/lang/Object;`
- `7: invokevirtual #13                 // Method left:(Ljava/lang/Object;Z)Ljava/lang/Object;`
**MethodHandleUse**（jar SHA `96bdcbc0b2b5962b800cdd64382466e4ee75d133e3d4d895887237df5288809a`）：
- `#40 = MethodHandle       5:#41          // REF_invokeVirtual MethodHandleUse.identity:(Ljava/lang/Object;)Ljava/lang/Object;`
- `#43 = MethodHandle       6:#44          // REF_invokeStatic java/lang/invoke/LambdaMetafactory.metafactory:(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodHandle;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/CallSite;`
- `1: invokedynamic #7,  0              // InvokeDynamic #0:apply:(LMethodHandleUse;)Ljava/util/function/Function;`
- `9: invokeinterface #11,  2           // InterfaceMethod java/util/function/Function.apply:(Ljava/lang/Object;)Ljava/lang/Object;`
- `BootstrapMethods:`
- `0: #43 REF_invokeStatic java/lang/invoke/LambdaMetafactory.metafactory:(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodHandle;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/CallSite;`
- `#40 REF_invokeVirtual MethodHandleUse.identity:(Ljava/lang/Object;)Ljava/lang/Object;`
**RawOwnReceiver**（jar SHA `f4fd34adc6ab76cbce3d87a2beb62d6375ddfa045ebf19322cfd1d07130b7ce2`）：
- `4: invokevirtual #7                  // Method identity:(Ljava/lang/Object;)Ljava/lang/Object;`
**ReboundOwnReceiver**（jar SHA `ee70661aec76415d5ca4a3c296502ef4128f44b820d1ececa58ab8f7811e86bd`）：
- `11: invokevirtual #9                  // Method identity:(Ljava/lang/Object;)Ljava/lang/Object;`
**InheritedUnknown**（jar SHA `4e6fff8b3902aee971cab85491b63d05411723be2ae9acb667a5fe5963ee4413`）：
- `2: invokevirtual #7                  // Method get:(I)Ljava/lang/Object;`
**VarargsCall**（jar SHA `93b793a3ec3fc6afd845e5d2231b0bdc484bf16dcc8960a45933c830481314c7`）：
- `flags: (0x0091) ACC_PUBLIC, ACC_FINAL, ACC_VARARGS`
- `9: invokevirtual #7                  // Method first:([Ljava/lang/Object;)Ljava/lang/Object;`
**SameErasureBinder**（jar SHA `6a4836c9be87000f4f6959415ce47b452ced7e25c64c79ca64de66ff3c572478`）：
- `2: invokevirtual #13                 // Method sink:(Ljava/lang/Number;)Ljava/lang/Number;`
**MultiUseResult**（jar SHA `08749c04b3dcce9459e131384ab38e40aeb1da1514aaf4c9f8a4eb0c99ef0cb3`）：
- `2: invokevirtual #13                 // Method identity:(Ljava/lang/Object;)Ljava/lang/Object;`
- `8: invokevirtual #17                 // Method observe:(Ljava/lang/Object;)V`
**BridgeUnknown**（jar SHA `350f04a0901b2988b9d46db166fba891ba1694e9f7a25a808f9c3fa0e7423ca0`）：
- `5: invokevirtual #9                  // Method apply:(Ljava/lang/String;)Ljava/lang/String;`
**IncompleteSite**（jar SHA `26c16dc2130a6a63193a0535016e519b2f048733d9b68cadef7950ba50bdb94f`）：
- `6: invokevirtual #7                  // Method identity:(Ljava/lang/Object;)Ljava/lang/Object;`

以上直接读取确认了输入 instruction/bootstrap/varargs 文本与冻结 jar 一致；它仍不提供 CLI 内部 AST/SSA proof trace，也不改变每个 family 的证据缺口或历史 root disposition。
