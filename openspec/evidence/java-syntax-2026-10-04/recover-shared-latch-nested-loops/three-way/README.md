# 三方行为对照（原 class / 固定 JADX dev / Jarde 重编），`java -Xverify:all`

编译器腿：原 class = 巡查冻结字节（SHA 见其 fixture-sha256.txt）；JADX dev =
`/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx`（渲染于
`jadx/S5.jadx.java` 等，`package defpackage;` 按 `defpackage.<C>` 运行）；Jarde 重编 = 本片修复后
`jarde-cli class-source` 渲染（`S5.fixed.txt` 等）经**真 javac 8**（Corretto 1.8.0_432）重编。

| 类 | 原 class | JADX dev 重编 | Jarde 重编 | 判定 |
| --- | --- | --- | --- | --- |
| S5 | `4685b821…` | `4685b821…` | `4685b821…` | 逐路径一致（`13/9/20/13`） |
| S3 | `472b7fe2…` | `472b7fe2…` | `472b7fe2…` | 逐路径一致（`[px:3]`/`[px:3, px:-1, px:5]`/`[a!, b!]`） |
| S4 | `7ef56628…` | `7ef56628…` | `7ef56628…` | 逐路径一致（`[k:v]`/`[c!]`/`[px:3, px:5]`） |
| ThreeLevel | `68ca3fba…` | —（本片变体，非三方锚） | `68ca3fba…` | 一致（`24`） |
| ExitDiverges / LabeledBreakOuter / Overlap | `1a252402…` / `4355a46b…` / `f807fe6d…` | — | —（拒绝形，无重编腿） | 拒绝边界，呈现见 `../negatives/` |

- `Svc`：原类行为 `orig.out`（`[px:3]`/`a=1;b=2;hits=1`/`true`）✓；lookup 体在 Jarde 渲染中完整呈现；
  整类重编腿被成员折叠通道的池形 `Svc$Entry` 拼写阻塞（javac 8 报"找不到符号 Svc$Entry"——
  `summarize` 引用未折叠成员，Non-Goal 域），`[px:3]` 行为腿由同形 `S3.nestedBreak` 承担。
- 三方输出逐字节 SHA 见 `output-sha256.txt`（含全部 `.out` 产物）。
