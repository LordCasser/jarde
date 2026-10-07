# 1.2 两行 `java.io` 宽化表行（javap 转录 + 自检 + 行集封闭性）

## 转录

同一 rt.jar（Corretto 1.8.0_432，`jre/lib/rt.jar` sha256
`b27515a608ee447566b688e2bbb2257b1f0d8eceb96c307b87eb28d90a6630f4`）、同一
`javap -classpath <rt.jar> <type>` 命令；转录行与协议文件更新在
[widening-row-sources](../../../evidence/java-syntax-2026-10-05/widening-row-sources/README.md)
（`javap-headers.txt` 末尾五行 + 两个构造位签名）。

| 表行（呈现类型 → 目标） | 实参位（javap 签名） | 依据（header 转录，逐字） |
| --- | --- | --- |
| `java.io.FileInputStream` → `java.io.InputStream` | `InputStreamReader(java.io.InputStream, java.lang.String)` 第 0 实参 | `public class java.io.FileInputStream extends java.io.InputStream {` |
| `java.io.InputStreamReader` → `java.io.Reader` | `BufferedReader(java.io.Reader)` 第 0 实参 | `public class java.io.InputStreamReader extends java.io.Reader {` |

对照（**不入表**，保守省略，逐字在案）：`java.io.BufferedReader extends java.io.Reader`（真实直接
边，但本片无其位点依据）、`java.io.InputStream implements java.io.Closeable`、
`java.io.Reader implements java.lang.Readable,java.io.Closeable`。`java.io.FileReader`
（`InputStreamReader` 子类）同样不入表。

## 自检

`sh results/selfcheck-javap-rows.sh` → `SELF-CHECK OK: the committed java.io lines are javap's own,
byte for byte`（对五个类型重跑 `javap`、按协议文件的索引列重排后与提交段逐字节 diff；两个构造位
签名另与 `javap` 输出逐字核对）。

## 行集封闭性（"每个实参位只由它自己那行到达"）

表单元测试
（`crates/jarde-java/src/build.rs::tests::platform_reference_argument_widening_reaches_exactly_the_java_util_table_ancestors`）
在既有正面/拒绝两列之外新增：

* 正面：两条新边各自可达（`FileInputStream→InputStream`、`InputStreamReader→Reader`）；
* 拒绝：反向（`InputStream→FileInputStream`、`Reader→InputStreamReader`）、跨接
  （`FileInputStream→Reader`、`InputStream→Reader`、`Reader→InputStream`）、未入表的邻居
  （`FileReader→Reader`、`BufferedReader→Reader`）、通配与 release≠8 全部保持拒绝。

因此 `WideningProbe.wrap` 的呈现只可能来自第一行、`buffer` 只可能来自第二行（无第二条路径），
"每行单独翻转自己的实参位"这一判定由渲染门控
（[03-component-gating.md](03-component-gating.md)：`cert+rows` = 1/1，其余 = 0/0）与上述封闭性
共同支持。

## 既有行/门/渲染零触碰

- 表内既有 java.util / java.lang Throwable / 其它族行**未改动**（纯增两行 + 文档注释扩写）；
- 渲染路径仍是既有的 `cast_argument`（表命中即写 `(目标类型)` cast），无新机制；
- 门控实验里所有既有宽化族 fixture（`p3-handlers`、`X3` 等）逐字节不变。
