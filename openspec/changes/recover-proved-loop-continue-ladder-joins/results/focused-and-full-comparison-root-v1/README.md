# 定向与完整源码对照验收

root逐一复开365个regular payload与inventory，核原始bytes、SHA和mode；压缩包SHA `004dd01900ed5b0fdb51ae59eb89985baab47b514267aac2a8eb40857553a2d7`，5031387B。原始基线及同轮IR另有已接受归档，不重复运行。

最终72命令：8 API、16次JDK8/23整类编译、32隔离VM共256观察全同原件，16错CodeSource根调用前拒绝；目标方法44/44物理BCI来源闭合，default/all除usage外不变。两个外置CLI逐字相同，64065312B/SHA6c83ec82cd4c6caee86c08e7fefddc2fea675952f936c0e4aa3f295ed9be7fc6；v5提供零源码变化构建，v4保留实际Clippy与指纹重建。

Java lib511/0、新公开2/0、旧复合更新7/0、嵌套数组8/0/2旧ignored、Reader1/0、指纹5/0/1旧ignored、fmt与完整CI Clippy接受。全部v1-v3真实失败与修正历史保留；绕过update及未批准operator控制仅parser/IR级，不冒称JVM verifier验收。target absent。两片共享fresh workspace/当前产品自身CI仍待，不关闭CF09或71/612。
