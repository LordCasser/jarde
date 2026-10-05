# 后缀自赋值健全性巡查（2026-10-05 root）——最严重级缺陷实证

## 发现：可编译但行为不同的文本（违反第一不变量）

三锚全部实证（渲染去注释后 javac exit 0、运行输出与原 class 不同）：

| 源 | 原行为 | 渲染行为 | 渲染文本 |
|---|---|---|---|
| `int i=5; i=i++; return i;` | **5** | **6** | `local0 = local0 + 1; return local0;` |
| `int i=5; i=i--; return i;` | **5** | **4** | `local0 = local0 - 1; return local0;` |
| `a[i]=i++` 后 `return a[1]*100+i` | **102** | **2** | 数组存语句**整条丢失**，`local0 = local0 + 1` 保留 |

机制：`iload_0(旧值); iinc; istore_0` 的 store 旧值来源证明失败时，引注只圈 store+load（`// @bytecode 6 2` + 诊断），但 **iinc 的语句仍以"已证明"身份呈现**（`local0 = local0 + 1`）——引注机制靠"文本碰巧不完整"保证不可编译（postOther 因缺 return 而不可编译），本族文本碰巧完整且语义错。**jadx 对照**：`int i = 5 + 1; return 5;`——行为正确（[results/jadx-SA.java](results/jadx-SA.java)）。

## 判别（同巡查健康面）

- `++i` 前缀（`i = ++i`）：正确（渲染多一个无害自赋 `local0 = local0`——行为 6=6，纯呈现瑕疵）；
- 语句位 `i++; ++i`：正确恢复；
- 跨变量 `j = i++`：**不可编译**（fallback 缺 return）——呈现缺陷级而非行为错级；
- 字段/方法同名（`foo`/`foo()`）、unicode 转义（`"aAb"`）、char 算术回赋（`(char)(c+1)`）：全健康。

## 归属

与在队 `recover-postfix-old-value-snapshot`（恢复片）**同族不同责**：恢复片负责让此形正确呈现；本片负责**恢复落地前的健全性守卫**——旧值来源证明失败的 store，其**整条语句**（含 iinc 的呈现）必须落入引注区，呈现文本不得可编译出不同行为。

## 处置

立项 `preserve-postfix-fallback-soundness`（窄、健全性优先）；恢复片 proposal 补三锚。
