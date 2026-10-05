# 字符串拼接极限巡查（2026-10-05 root，负结果——常量折叠全边界过）

## 探针

[fixture/CS.java](fixture/CS.java)（`--release 8`）：`"pre"+null+"post"`（null 拼接——`"null"` 字面量化但**不折叠**）、char 序列、全类型混合、`1+2+"=3"`（**int 先算后接**）、`"1"+2+3`（String 先行全接）、自引用拼接 `s = s + "b" + s`。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- **null 拼接**：呈现 `"pre" + null + "post"`——javac 对含 null 的拼接**不折叠**（运行时才知 null），保留 `+ null +` 形如实；
- **求值序边界精确**：`1+2+"=3"` 呈现 `"3=3"`（int 算术先折叠再接——`1+2` 是 int 表达式先算）；`"1"+2+3` 呈现 `"123"`（左结合 String 先行——全部拼接）——**两个方向的折叠边界都精确**；
- **自引用拼接**：`local0 + "b" + local0` 保留（运行时依赖不可折叠）——区分折叠/不折叠的判据精确；
- 行为 `prenullpost/xy/n=1 c=c b=true nul=null/3=3/123/aba` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。字符串拼接折叠全边界（null/求值序/自引用）确认覆盖。
