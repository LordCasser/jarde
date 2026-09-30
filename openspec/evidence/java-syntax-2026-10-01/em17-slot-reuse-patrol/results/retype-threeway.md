# A2 与变体三方对照（recover-array-slot-retype-locals，变更后）

日期 2026-10-01；JDK 23（`javac --release 8` 编译所有 Java 输入）；`java -Xverify:all`。
固定 JADX：`/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx`
（默认参数；参照不作为语义正例，见巡查 README）。
Jarde：本 worktree 变更后 `target/debug/jarde-cli class-source --input <X>.class --policy single-class --class <X>`。

冻结输入：`fixture/A1.class`、`fixture/A2.class`（SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）；
变体 `V1`–`V4` class 与恢复基线冻结于仓库 `tests/fixtures/p3-array-slot-retype-locals/`
（v8 class + baseline + expected，摘要随全库 fixture 指纹
`tests/fixtures/corpus-fingerprint.json` 固定）。V1–V4 源码由
`javac --release 8 -g:none` 编译（与 fixture 同为无调试信息的 verifier 有效类）。

## 行为对照（各方 main 正常路径，`-Xverify:all` 无告警、退出码 0）

| 输入 | 原 class | 固定 JADX → javac 8 重编 | Jarde（变更后）→ javac 8 重编 |
| --- | --- | --- | --- |
| A2（int[]→boolean[] 两段） | `2,3,4,true` | `2,3,4,true` | `2,3,4,true` |
| A1（动态维度/混合初始化器） | `2:3:3` + `3:5` | 同左 | 同左 |
| V1（三段交替 int[]/boolean[]/Object[]） | `2,3,4,truey` | `2,3,4,truey` | `2,3,4,truey` |
| V2（同型多定义） | `3` | `3` | `3` |
| V3（汇合点 phi 合流，真别名） | `[I` | `[I` | 维持既有呈现，`javac 8` 拒绝（`boolean[]无法转换为int[]`；变更前后同，负例逐字不变） |
| V4（循环携带 phi 合流） | `[I` | JADX 自身输出不可编译（`Object obj = {1, 2, 3};` 非法初始化器） | 维持既有呈现，`javac 8` 拒绝（同 V3；变更前后同，负例逐字不变） |

JADX 参照限制照录：V4 的 JADX 恢复文本不能重编（其 `Object` 局部上的数组初始化器拼写），
与既往巡查记录一致，不影响三方判定（Jarde 各负例与自身变更前逐字一致即为该行的验收）。

## 命中输出（Jarde 变更后，fillCalc/three 成员节选）

A2.fillCalc（段二新名 `local2_2`，段一声明与拷贝按既有规则保留）：

```java
int[] local2;
int[] local0 = new int[]{side(), side() + 1, side() * 2};
local1 = new java.lang.StringBuilder();
local2 = local0;
for (int local5 : local2) { local1.append(local5).append(','); }
boolean[] local2_2 = new boolean[3];
local2_2[side() - 1] = true;
return new java.lang.StringBuilder().append((java.lang.String) local1.toString()).append(local2_2[1]).toString();
```

V1.three：`int[] local1 = new int[]{…}`、`boolean[] local1_2 = new boolean[3];`、
`java.lang.Object[] local1_3 = new java.lang.Object[]{"x", "y"};` 三段自有声明。

## SHA-256

```
24a58f9a4f5a5c2d0e4e5cce4f9bf2bc404aa81591a55e9eaacf0e53a2e8ced0  fixture/A1.class（原 class）
0e441de3da38795dd1c9495e7d6d34972a4bc7c90e49274fb63c6869c6163e35  fixture/A2.class（原 class）
7716b76c8e63ec2c254305ffad41700ef42784e038c76bf32150a9d039e1bbfc  tests/fixtures/…/v8/V1.class
22d88380a39d0080fab5ff902e4ba7366c74f0a97f1cda76663d699390b63a37  tests/fixtures/…/v8/V2.class
e48f86634de9044cf0fa6689e9929c6b3ba78a109bb6ad2ba0846e83a2b6fdc6  tests/fixtures/…/v8/V3.class
e85de3405f70237686ae10ac2168e0792e17c3606544a53c9e5dfd4745bd34e0  tests/fixtures/…/v8/V4.class
6844af182c14d028418702b1daa05f0a19f688902634b9f5bc5f6ef2d420b37d  results/A2-retype.before.java（变更前恢复）
db24a02993905bd5ebbdc199927604197319356584e54224968affa866fa588e  results/A2-retype.after.java（变更后恢复）
ca9109e99a8f8e97dec77739ba66d41f5650699468284f0bd2753363a3aba1a2  results/A1-retype.after.java（= 变更前记录，逐字不变）
22370cf44b9bd6a169ceab81dc09eaaad3f6b80eaeec765236b2b21df2f786a5  variants/retype-V1.before.java
de51423bac3c2847c7114fb999c49dd6c688c12eb814134894ed3a3cf709e5ce  variants/retype-V1.after.java
a8a2f7dfe78259a566fcd9054742ecadfb410b653e5b96cee38651693912c4ed  variants/retype-V2.before.java（= after，逐字不变）
959dce5c0bddfa5e886e9460ac62c2280ccfb9c125293518ee60602cf05983f6  variants/retype-V3.before.java（= after，逐字不变）
2a275195295cb73c3cb37d7e555bfb99d82e404fccaf098eb22a7d2f91449074  variants/retype-V4.before.java（= after，逐字不变）
a0efe7bb2e43d3824ec95a69bee08aeb816a3c2ae2414c30536800cb305d92c3  results/retype-jadx-A2.java（JADX 参照）
fb2647c0603bb87866d7db8ef3791ecd0743291dadec360e1d23eea118ed7474  results/retype-jadx-A1.java
15cbfeec9306ad912bdbc6c027d09ea6ea10221bf207bb1ea3b7d93048fb83fd  results/retype-jadx-V1.java
29215657dfabcd88af7548e97fe2010e43cfc94fcbf2ce65c8e6b1e0473d0cea  results/retype-jadx-V2.java
7056484e9d32e889f96104edf7a3ec8ba4e37e653e1012e804c6c7de64d361f5  results/retype-jadx-V3.java
be2e492b6e0ecbb8aa92f26f3e1e89cdcdea75e22bade8b7f79ee084fb5c5a60  results/retype-jadx-V4.java
```

命令（可重放）：

```
javac --release 8 -g:none -Xlint:-options V1.java                # V2–V4 同法；fixture class 已冻结
java -Xverify:all -cp . <X>                                       # 原 class 与各方重编产物分别执行
<JARDE> class-source --input <X>.class --policy single-class --class <X>
jadx -d <out> <X>.class                                           # 固定 JADX，默认参数
```
