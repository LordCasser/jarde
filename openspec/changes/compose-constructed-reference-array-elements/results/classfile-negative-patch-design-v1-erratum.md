# `classfile-negative-patch-design-v1.md` 勘误

原报告 SHA-256 为 `982aeb70cf35cb78e1f8fe28ee724238a4c0444faea5998ac9c5f889ede6de46`；root 已读过该版本。正文第 82 行的栈峰值计算正确：原始峰值为 6 slots，`dup_x2` 变体峰值为 5 slots，故 `max_stack` 无需增长。第 118 行第 2 点却误写为“需要 `+1`”，与前述计算矛盾。

已只更正该句：patch 前检查实际 `max_stack >= 5`；本冻结方法原值为 6，变体无需增加 `max_stack`。Code 长度和 Code attribute 长度仍各增加 2，后续 BCI 随之偏移。没有重写或隐去历史版本。

更正后报告 SHA-256：`253ee42cfcf40b53c21dba24f6d9642087e2b45bd5bf9670845516e562cd4fad`。
