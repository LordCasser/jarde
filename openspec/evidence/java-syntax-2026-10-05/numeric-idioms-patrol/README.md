# 经典数值惯用法集巡查（2026-10-05 root）——XOR swap = family 3 新形状；其余健康

## 结果

- **min3/max3/clamp 嵌套三元、分支 abs、gcd（循环内 temp 三步赋值）全恢复**（quotes=0）；
- **xorSwap QUOTED = 已知 dependency-chain 族（family 3）verbatim 诊断**：`p[0]^=p[1]` 三链接力（每步读上一步写）——数组元素复合 RMW 链形；空 body + 非 void `int[]` 缺 return → **编译失败 = SAFE 响亮拒绝**；jadx 平凡解（三行 ^= 逐条）；
- main 12 quotes = xorSwap 拒的级联。

## 处置

**soundness 片 family 3 补充数据点**（不新立）：XOR swap 是 family 3 的最高频经典形状（面试/图形代码常见）；守卫落地后它保持 SAFE；恢复侧归 compound/RMW 域队列。数值惯用法域（三元 min-max-clamp/abs/gcd）确认覆盖。
