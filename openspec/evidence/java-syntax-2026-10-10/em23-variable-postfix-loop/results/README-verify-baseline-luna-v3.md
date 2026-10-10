# EM-23 变量后缀循环基线独立核验 v3

`verify-baseline-luna-v3.py` 只读取已封存的 `baseline-root-v1` 文件，不启动
Java、JADX、Jarde、Git 或 Cargo。执行时显式传入冻结清单与 inventory 的哈希：

```sh
python3 openspec/evidence/java-syntax-2026-10-10/em23-variable-postfix-loop/results/verify-baseline-luna-v3.py \
  --manifest-sha256 e8aec7964455ab84ffb12d984140e18bffe94e5cc0cf7f1eb5de06c21cc23a2d \
  --inventory-sha256 89af5a986896541c5adb0dd8bed30c89d61038017f05d57634142e8416cce783
```

本次基线的实际原程序输出为：`null=0`、`empty=0`、`mixed=2`、`repeat=1`、
`null-element=NullPointerException`。核验要求原始 JDK 运行流精确匹配该值，并要求
同一 JDK 的 JADX 两种配置与原程序三流相同。

核验成功只表示证据与观察结果吻合，不表示 Jarde 语法恢复成功。四份 Jarde 报告
均保留完整物理成员；`countEmpty(Ljava/util/List;)I` 的结果是
`explanation_only`，fallback 为 `jre_region_arms_do_not_meet`。四份未经修改的
完整生成类都在 javac 阶段以“缺少返回语句”失败，因此没有 runtime 或 class 输出。
这是本基线记录的负结果。

v3 还逐 leg 比较 default/all 的整类文本、每个物理方法文本和 source map，并以
`primary` 与 `derived` 两类来源核对原方法身份及全部 javap BCI。它没有通过
source-map 证据推断 fallback 文本具备 Java 语义。
