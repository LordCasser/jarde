# EM-12 主线独立验收

Root 将实现提交 `2c36ee674e2cec388f0e1e733b02cf88a653c191` 拣入主线为 `ad617d09` 后，重新构建 `jarde-cli`，在新的空目录独立执行 [replay.py](replay.py)。脚本固定 JADX 修订和七个测试/实现文件哈希；普通及重载两套的原 class、JADX 与 Jarde **完整 Java 8 源码**分别重编并以 `java -Xverify:all` 运行，输出分别为 `20:10:1:3` 与 `number`。两套 Jarde 生成源码与提交的 [`after/source/jarde`](after/source/jarde) 和 [`after/binding/source/jarde`](after/binding/source/jarde) 逐字节相同，家族投影均为 `projected`。`member_family_identity` 的两个 `outer_super` 目标测试及 OpenSpec strict 验证通过。

实现只放行普通单引用参数直接载入且竞争参数经所选完整 class 父链证明为严格子类的情况。泛型、接口、数组、varargs、装箱、缺类和可能适用的重载仍拒绝；这里不推定 EM-12 全部变体已追平。
