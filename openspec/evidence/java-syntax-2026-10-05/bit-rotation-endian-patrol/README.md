# 位旋转/字节序/十六进制巡查（2026-10-05 root，负结果——系统代码高频全过）

## 探针

[fixture/BR.java](fixture/BR.java)（`--release 8`）：**查表 hex 转换**（`HEX[(v >>> (i*4)) & 0xF]`——clinit `toCharArray()` 字段初始化 + 移位掩码下标）、`Integer.rotateLeft/reverseBytes/bitCount`（intrinsic 候选）、byte 序列 `Character.forDigit((b>>4)&0xF, 16)` 十六进制序列化。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- 查表 hex：`HEX = "…".toCharArray()` 字段初始化器、for 递减循环、`BR.HEX[arg0 >>> local2 * 4 & 0xF]`（注意 `>>>`/`&` 优先级呈现即 javac 物理序——括号层级正确）全部精确；
- rotateLeft/reverseBytes/bitCount 直接调用保留（intrinsics 在 javac8 只是普通 invokestatic，无内联——物理事实）；forDigit 链恢复；
- 行为 `deadbeef/-16711936/1144201745/20/cafe` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。位旋转/字节序/hex 族确认覆盖。
