# 任务 1.2 / 1.3 / 1.5 的"实现前 / 实现后"逐格记录

```sh
sh results/04-baseline/render-cells.sh /tmp/pih-bin/jarde-cli-base   > results/04-baseline/before.txt
sh results/04-baseline/render-cells.sh /tmp/pih-bin/jarde-cli-patched > results/04-baseline/after.txt
diff before.txt after.txt
```

二进制：base = 本片改动前的 `cargo build --release -p jarde-cli`（sha256
`37027c690fe634149a7346749f3b0b0f7a89e21b06f1ff44acc5bf922d0bb247`）；patched = 本片改动后
（sha256 见 `results/06-gates.md`）。两次跑的是**同一批 fixture 字节**（脚本先自检：真渲染必须带
`// jarde: presentation of` 自述头、不存在的类名不得被当作源码应答）。

`diff` 的完整形态（13 格）只有下列 7 格移动，其余 6 格逐字未变：

| 格 | before | after |
| --- | --- | --- |
| `iface-single` | `implements java.lang.Comparable`，桥 visible | `implements java.lang.Comparable<IfaceImpl>`，桥 hidden |
| `iface-multi` | `implements java.lang.Comparable, java.lang.Runnable`，桥 visible | `implements java.lang.Comparable<MultiIface>, java.lang.Runnable`，桥 hidden |
| `iface-erased` | 同 iface-single（裸头 + 桥 visible） | `implements java.lang.Comparable<ErasedCall>`，桥 hidden（body 实参边界见 03-fixtures） |
| `iface-arity` | 裸头、无 header 拒绝 | 新增 header 拒绝 `an interface definition contradicts the class Signature's type arguments`（裸头文本不变） |
| `br-impl` | `implements java.lang.Comparable`，桥 visible | `implements java.lang.Comparable<BR$Impl>`，桥 hidden |
| `br-strbox` | `extends BR$Box` + header 拒绝 + `set` 桥 visible | `extends BR$Box<java.lang.String>` + 无 header 拒绝 + 两条桥都 hidden |
| `parent-spec` | `extends Outer$Box` + header 拒绝 + `set` 桥 visible | `extends Outer$Box<java.lang.String>` + 无 header 拒绝 + 两条桥都 hidden |
| `iface-unresolved`（不变） | `implements MissingApi` 裸、桥 visible（unresolved 拒绝文本） | 逐字相同 |
| `iface-typeuse`（不变） | `implements java.lang.Comparable` 裸、桥 visible（接口边前置文本） | 逐字相同 |
| `parent-criteria-nested`（不变） | 裸头 + `direct superclass does not resolve…` 拒绝 | 逐字相同 |
| `parent-criteria-bare`（不变） | `extends NB$Box` 裸、无拒绝 | 逐字相同 |
| `parent-arity`（不变） | 裸头 + `direct superclass does not resolve…` 拒绝 | 逐字相同 |
| `parent-multiseg`（不变） | 裸头 + `only a single direct Parent<String> superclass…` 拒绝 | 逐字相同 |
