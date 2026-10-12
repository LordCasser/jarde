# 引用 Load 回退修复与最终局部验收

完整workspace-v5发现捕获参数经别名写入Runnable局部，原SHARED/ESCAPED拒绝断言保持。最终修复复用有预算的catch SSA别名证明，拒绝空phi/direct Caught；新引用Load逐写要求精确named verifier类型或完整限定java.lang.Object，简名Object和Unknown不能冒充精确赋值证明。四lane foreach及Stop门保留，无新solver/public carrier。

final-check-v9实际8命令全0：当前CI Clippy、391库测试、8integration targets共122成功/4 ignored、fmt/build/global strict和finallyexactclean；4854来源前后相等，CLI SHA947d6dd29a2f8e0442fc848c8ab24da2a292ea54d0ca54e574a4be26c78059d2。测试后局部cargo clean保持target1GiB，不提高上限；v7真实资源失败及v8历史来源保留。CLI二进制不重复打包，原冻结文件仍保留。

最终parent-v5为40命令/16renders/12未修改完整类编译运行，字段-v4为24命令/4Jarde+4JADX完整类，TWR-v3为23命令/原-JADX-父项-当前双JDK：必需运行均同原。TWR旧checker钉旧CLI路径导致的失败原样保留，新root-v4 checker只更正当前路径并接受同一实际raw，不重跑或拼接样本。匿名-v3为29命令：原/JADX行为同原，当前3物理类整体仍保留构造器编译债务/无runtime；3份当前class source与先前候选逐字相等，root/Base既有投影证据仅按相同源码范围复用，不宣称整体通过。

归档包含root重开核验的每成员bytes/SHA、原始stdout/stderr、源码/fixture恢复结果、来源与停止检查及私有候选/审查/工具。完整workspace双seed正在独立fresh运行，尚未接受；本归档不证明全工作区、自身CI、产品推送或干净main交付。
