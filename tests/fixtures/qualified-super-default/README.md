# qualified-super-default fixture：限定 super 默认调用

`NestedSuper.java`（默认包）和 `pkg/PackedSuper.java`（包内）固定 `invokespecial InterfaceMethod`
限定 super 默认调用的可恢复形态；`NestedSuper$Indirect`/`NestedSuper$Abstract` 是两个负例的**合法邻居**
（`javac --release 8` 拒绝编译"限定符仅经 extends 继承"与"目标 abstract"两种源形，故测试内对合法
class 做单索引常量池补丁，见 `tests/qualified_super_default.rs` 的补丁函数与断言）。

```text
javac --release 8 -g:none -d v8 NestedSuper.java pkg/PackedSuper.java
```

原类运行事实（`java -cp v8 NestedSuper` / `java -cp v8 pkg.PackedSuper`）：

```text
AB
I:A
log:hi
g7
A
A
PI
```

| class | SHA-256 |
| --- | --- |
| `NestedSuper.class` | `dad93492bbae25b41a9de5e1bd7dee6fb651d4eca19e4dfecfdfd563d88867b7` |
| `NestedSuper$A.class` | `c1b8d5bf626d018ae96bc71f69968e86025c81b2e3dd8cb597e61e22d1d79880` |
| `NestedSuper$Abs.class` | `4d54d68aca34813fe3b06a2ece2b714f6a97cc37f37cf215206c06c955fd5348` |
| `NestedSuper$Abstract.class` | `8f6f2efa95f5338c598844409c8eaaec2fe73850b6090dcc6beff15e3ff3a4c8` |
| `NestedSuper$B.class` | `840f381583b68ed2cb86d7b87a372e3cd7aeaa449d1fc785fa1e4588453e4cec` |
| `NestedSuper$Diamond.class` | `f3fdf28b731d88571072aa6220164c0173deadfcd064b48c0056715f340d0333` |
| `NestedSuper$Indirect.class` | `10a858f6f8d8c1e274f0e09fbcf78178d7d841eafcbe0bd90dd3917efede863b` |
| `NestedSuper$Single.class` | `ccf2c0cc2d5a00280f21ec349570a0e17e075064565c48e7b46ff3190baabbe3` |
| `NestedSuper$Sub.class` | `528e9fa9e9dbb1be60c15bf83e792e9ae965211abae1de833eb8eb717bd94b7d` |
| `pkg/PackedSuper.class` | `ecf597e43c9ff076cdfce951f88e5f25ad900d1328be0bb72d4e2daf568ce9c1` |
| `pkg/PackedSuper$I.class` | `559b7c1cbd6393b0efaf5c6df709746ee77702c47ed801a27656cf3d975c57c0` |
| `pkg/PackedSuper$Use.class` | `02ee50bf40f34aa6c3ce12643698fe1600d5aa0b8e801c0eabc36b18f04837b0` |

负例补丁后的字节不是 javac 产物：`NestedSuper$Indirect`（接口槽 `A`→`Sub`）连 HotSpot 校验器也
拒绝（"Bad invokespecial instruction: interface method reference is in an indirect superinterface"）；
`NestedSuper$Abstract`（调用属主 `A`→`Abs`）可通过 `-Xverify:all` 链接但调用会抛 `AbstractMethodError`。
两者只作为字节输入驱动拒绝断言。
