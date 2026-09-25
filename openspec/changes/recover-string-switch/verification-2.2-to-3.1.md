# 字符串 switch 结构投影与整类对照（2026-09-24）

`region::recover` 先各自完成两层普通 switch 和覆盖检查。投影仅在同方法证书、区域相邻、join/最终入口一致、两层原始 BCI 所有权与标签映射全部闭合后，用一个 `Region::StringSwitch` 原子替换这两个区域；证明拒绝时原区域不变。构建层只呈现一次 selector，沿最终 arm 及其穿透顺序生成 Java `switch (String)`；AST 以封闭的 enum/String 展示标签保留原整数 key，不另建打印器。JADX 的先改指令后尝试替换区域、全变量清理和未映射整数 key 拒绝路径没有沿用。对于与 default 同入口且已证明不可达的整数空洞，Jarde 能折叠 `StringSwitchMiddleDefault`，JADX 1.5.6 仍输出两层。

Root 使用代理保存的最终 CLI `/tmp/jarde-string-projection-replay/jarde-cli`，SHA-256 `d3bd76e99733d3d7be66559564e0f0f22452a9b156e1e1ce6744d519c98ee1eb`，独立对冻结 class 运行 `class-source --policy single-class --release 8 --format text`。每份输出**原样**以 `javac --release 8` 重编完整类，再用同一 source-only runner 和 `java -Xverify:all` 对照原 class；另从当前 JADX 1.5.6 重新生成源码并以相同方式执行，唯一修改是删除其自动插入的 `package defpackage;` 行。

| 冻结输入 | 原 class SHA-256 | 行数与原/JADX/Jarde 输出 SHA-256 | 新 Jarde / JADX 形态 |
| --- | --- | --- | --- |
| `StringSwitchProbe` 碰撞、一次 selector、null | `258fea472bfd8f80ee49726dc25f0c3d46dce8f61db57d43d5f63e52e6c2c53c` | 10，`deadce3d716d782f6894aebcbb0652e231a10fbb13402b7a94a93d4d58b6a789` | 1 / 1 层 |
| `StringSwitchUnicode` 空串、BMP、代理对、null | `434e6e84a045d67bc0a6337d89f5493779e12dc01133d89355220ff57cf367fa` | 5，`41489c44ab3e0c532afb12365f3b396f19a5e6ecf809a1233feaf01e14750a95` | 1 / 1 层 |
| `StringSwitchMiddleDefault` default 空洞与穿透 | `fd6caaaac6c00799e218601c76a6dc133238a851d05672237219bfbebfb2a647` | 18，`45960ba35edf1da3f6f268c61e4b5d7e2cb2026ddd7799150089aa6c00b0e124` | 1 / 2 层 |
| `StringSwitchVariants` 共享 case、穿透 | 本次同源 Java 8 重编 `f994a3b8a50a2b535fb1bf2eecf741766bddaad411fd47b0ed8d367707283be6` | 18，`6e8876ff4e0590db16accb6b09585b42b9cc65d5aae3b02d3533beb58db2a79d` | 1 / 1 层 |

负例也以最终 CLI 独立重放：`ExtraHashUse` 的额外 hash 消费、`WrongHashBucket` 的错误 literal hash，以及 `ExtraBucketEffect` 的桶内 `calls++`，Jarde 均保留两层普通整数 switch，完整类 Java 8 编译成功。前两者共用 runner 的 10 行、额外 hash 独立 runner 的 6 行、桶内副作用 runner 的 4 行，原/Jarde 逐字相同；`ExtraBucketEffect` 的 `Aa:10:calls=1` 与 `BB:20:calls=1` 证明副作用未被误删。JADX 对 `WrongHashBucket` 和 `ExtraBucketEffect` 保留两层并可编译执行；`ExtraHashUse` 的 JADX 错误折叠会输出未声明的 `r0`，详见[原始负例证据](../../evidence/java-syntax-2026-09-24/string-switch-negative-fixtures/analysis.md)。

代理的集成测试 2/2、Java 包单测 139/139、enum 投影 2/2 及整数/char 相邻回归通过，`cargo fmt --all --check`、`git diff --check` 与 `openspec validate recover-string-switch --strict` 通过；专用 Cargo target 已清理。Root 的独立整类结果确认 2.2–2.3 与 3.1 的行为和形态范围。2.4 仍须检查每个被折叠 BCI 的来源、预算、取消以及 essential/all 的一致性。另有一个需要在 2.4 审查的保守退路：证书成功但构建层随后无法呈现 selector 时，当前会引用整个新结构，而不是恢复原来的两个普通区域；应确认这不会比未投影时丢失可执行源码，必要时把可呈现性纳入投影准入。
