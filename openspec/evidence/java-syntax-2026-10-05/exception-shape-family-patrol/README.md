# 异常形家族巡查（2026-10-05 root）

## 探针

[fixture/EX.java](fixture/EX.java)（`--release 8`）：multi-catch `catch(A|B)`、**finally 内 return**（吞 try 值）、finally 副作用+正常抛出、finally 嵌 finally。

## 结果

| 形 | 呈现 | 判定 |
| --- | --- | --- |
| multi-catch | `catch (ISE \| NPE local1)` **完整恢复** + 体内 `e.getClass().getSimpleName()` | **健康** |
| finally 内 return | `if (n==0) return -1; else return local1;` + 3 弃置块注记 | **行为等价**（[FR 判别探针](fixture/FR.java)：6 值全一致 `-2..3`——`n==0→-1`、否则 `n`，与原 try/finally 语义逐值相同）；呈现为 if/else 是有条件等价退化（同 multi-catch 巡查的 continue→break 先例） |
| finally 副作用+抛出 | if/else 双臂带 BCI 注释（"exceptional path repeats code…"）+ 1 弃置块注记 | **部分呈现**（引注形——结构未全证但文本如实标注；finally 的 println 复制在注释中说明） |
| finally 嵌 finally | 整方法拒绝 | 归 **local-scope 域**（"local 1 crosses a quoted fallback region"，与 NA-b 同因同登记） |

## 处置

- multi-catch 与 finally-return 等价退化：**不立项**（负结果 + 等价证据）；
- finally 副作用部分呈现：引注形合规（响亮），呈现质量债数据点；
- finally 嵌 finally：归并进 `preserve-local-scope-across-exception-regions`（其锚家族 +1：NA-b、EX.nestedFin 同因）——在该 change 的 Why 已有 root 数据点段补一行。
