# char[] 手工字符串处理巡查（2026-10-05 root，负结果——就地字符串操作全过）

## 探针

[fixture/CH.java](fixture/CH.java)（`--release 8`）：`toCharArray` + **双指针 swap**（for 双变量 `i++, j--` + temp 交换 char[] 元素）+ `new String(char[])`；**run-length 压缩**（SB + 内层 while 短路条件 + 双指针 j 推进 + 条件 append）；双指针**回文早退**（不等 return false）。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- reverse：`local2 = 0; local3 = local1.length - 1; while(local2 < local3){ temp swap; ++/--; }` 完整——**for 双变量 update 正确归一 while+双语句**（#71 结论在本复合形再证）；
- compress：嵌套 while（短路 `j < len && cs[j]==cs[i]`）、`if(j-i>1) sb.append(j-i);` 条件 append、外层 i=j 跳跃全精确；
- isPalindrome：双指针不等早退+else 双递进完整；
- 行为 `olleh/3ab2c/true/false` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。char[] 就地操作族（swap/压缩/回文）确认覆盖。
