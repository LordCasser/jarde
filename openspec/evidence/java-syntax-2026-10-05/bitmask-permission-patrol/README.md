# 位域权限系统巡查（2026-10-05 root，负结果）

## 探针

[fixture/PM.java](fixture/PM.java)（`--release 8`）：static final 掩码族（`1<<n` 四常量）、grant（`|`）/revoke（`& ~p`）/can（`& != 0`）/canAll（`& == ps`）三件套+全测、位移生成（`1 << ord`）、main 多次 grant/revoke 累积。

## 结果：**健康，无缺口**（quotes=0）

- 五方法全恢复；**判别点补格**：#99 锐化的运算符维度只对**实例复合 RMW**（`this.flags |= …`）成立——**纯表达式位运算**（参数位 `cur | p`、`cur & p`）全恢复，失败面进一步收窄到 receiver-dup 复合赋值形；
- **`~p` 呈现为 `p ^ -1`**：javac 把按位取反发射为 xor -1，jarde 呈现物理指令形——语义恒等（-x-1 = ~x），忠实呈现域（非差异）；
- 掩码常量在 main 中按 javac 常量折叠内联（`READ | EXEC` → 字面量）——与常量族结论一致；
- 行为 `true/false/true/8/3` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。
