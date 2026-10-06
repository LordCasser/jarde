# `recover-parameterized-interface-headers` 的父类腿 fixture

变更 `recover-parameterized-interface-headers` 并入的父类池形参数化 MVP（放宽两处 `$` 拒绝）。这些格是
**负例与对照**：正例（`Spec`、`BR$StrBox`）沿用 `tests/fixtures/p3-bridge-projection/` 的既有冻结族。
`api1/` 是 `ArityExtends` 真正编译时依赖的 1 形参 `NB$Twin`（只用于编译），`api2/` 是环境里那一份 2 形参
定义（拒绝格的成因）。构建脚本：`…/results/03-fixtures/build-fixtures.sh`。

| 类 | 声明（自身 `Signature`） | 期望 |
| --- | --- | --- |
| `NestedExtends` | `extends NB$Box<java.lang.String>`，而 `NB$Box extends NB$Root` | 保持拒绝：`direct superclass does not resolve to one proved single-parameter parent definition` |
| `ArityExtends` | `extends NB$Twin<java.lang.String>`（环境装 2 形参 `NB$Twin`） | 保持拒绝（同上文本） |
| `Multiseg` | `extends MO<java.lang.String>.Mid`（多段路径，`$` 连接 `MO$Mid`） | 保持既有拒绝：`only a single direct Parent<String> superclass is supported…` |
| `BareBox` | `extends NB$Box`（无实参） | 无投影、无拒绝：裸头逐字不变（普查对照组） |
| `NB` / `NB$Root` / `NB$Box` | 父类族 | 供上面两格的解析 |
| `MO` / `MO$Mid` | 内部类族 | 供多段路径格 |

`NestedExtends`/`BareBox` 引用 `NB$Box`、`ArityExtends` 引用 `NB$Twin` 时都**只经 classpath 的 class 文件**
解析（同一 javac 运行里 flat 名只能被一个文件引用一次，见 change 证据 02-q2-replay 的记录）。

## class 文件 SHA-256

| 文件 | sha256 |
| --- | --- |
| `v8/NB.class` | `5550c76b1a0e4cdeec7ad338e082a00ab8c91afd92a82e45e29b940e618e6af1` |
| `v8/NB$Root.class` | `9d58a26b40a62c66385b06c3d9820f4f0ded0292ab65b26f3e7bbf2b5ef2a49d` |
| `v8/NB$Box.class` | `19a9f6bc9c3343844bf920acf09141a32313a4d11fe0617869af8306d3c7e319` |
| `v8/NestedExtends.class` | `532650c6ebb18e8289c50fdc84d4f4b8cf5b02fe23c93f060c8ce05cffaab674` |
| `v8/BareBox.class` | `38d614cbec056baba9c36fa572b5d4cc21b75c7c2be3cdf82cbe04c69aac14aa` |
| `v8/NB$Twin.class`（api2，2 形参） | `2436650740bd8661748f6814f59fec88da8a56af624ca8d5ef881f4f859c95dc` |
| `v8/ArityExtends.class` | `88dd571038ace141c8f5bb5e8cdc06d86bb6e5f4101cb3d8b16c622fd01517c8` |
| `v8/MO.class` | `c5bc207cc0c40c72f0797e6a2073691f96874eab1caf9077f923574bc7055c8a` |
| `v8/MO$Mid.class` | `f1e4b730c016fdc468b9b6ab35f3e933d9ae7444cb13a30c149c974881294c8a` |
| `v8/Multiseg.class` | `532a3bd400253eb159a9bbdc0f7f99201014ab07104678d4238dbe20b56a36c0` |
| `v8-javac8/NB.class` | `d5ec8d31dfc6112eede36a0ed72d75f31447be9bba0fd488af3ef4622afef6cb` |
| `v8-javac8/NB$Root.class` | `c6a902bc7924f77095588fb88dab7b5546fd95771e941538095d16d2a09a728c` |
| `v8-javac8/NB$Box.class` | `703a9d1686003419e94f469436cff296a5964b16ed439ecc7d21dc4f27a6e698` |
| `v8-javac8/NestedExtends.class` | `11baf2659c3027af5118e38b014e64520fe7856c0e7a8387fd9a0d19fb57ab03` |
| `v8-javac8/BareBox.class` | `aac60a9d8137e320c833a9ea206a9d1ff44eb7c1634f1fc1baed72fb78ca83c7` |
| `v8-javac8/NB$Twin.class`（api2，2 形参） | `75f746a38cd92e2b6793333d30dc8042d451f141dbb709c2a79498c71359ddc2` |
| `v8-javac8/ArityExtends.class` | `0960f678a8c1e43407ece39efa7077a62e87b89c01b5e866e596e833aa68d17e` |
| `v8-javac8/MO.class` | `76a6ead005fc947b0e1e2e47b1396fef52f2e9b8d90bbef6c2df566029fa1209` |
| `v8-javac8/MO$Mid.class` | `774c52fb77542fc533acec59b6da1aad7f7670899d9770cbc3b87ae573568a49` |
| `v8-javac8/Multiseg.class` | `382fca62a5f2ecdcb66d5f5b4fe828dc0062e2fa329194bc5371577e1572fe83` |
