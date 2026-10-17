# 父级剩余形与物理来源取证

本包只保存新增六命令控制和纯Python物理来源核对，不改变产品、不产生新的rebuilt运行。capture.tar.gz为2,470,518B，SHA256 `835ff092be6439c00b2273122bce49f4833075d064ed37c50c1c9453be44ffb8`，62payload+manifest共63个regular成员。外置manifest逐字等内嵌；每项含相对path/hash及原始provenance path，绝对路径需要按相对成员映射。CLI/JDK二进制未嵌入，收据保留pins；这不是完整仓库或离线工具链。

- return-controls：当前05e与前一b4a6 CLI的EX.finReturn方法文本相同且仍有uncovered-block fallback；原样FinallyCompletion两编译器class各907B/v52/7个Code方法，完整生成源码均4114B，各javac失败3处missing return。六命令实际exit为0/0/0/1/0/1，12raw保留，0 rebuilt运行。79项独审无错误；历史FinallyCompletion原source/Runner/215B oracle pins不变。EX.java未纳本次inputseal，不冒称其本次动态前后hash已证明。
- physical-origins：对最终FY/FX/FO六整类32方法独立解析724个物理指令实例，报告来源没有缺失/多余且方法身份相符；209项独审无错误。这是来源核对，不是新行为实验。
- 原始输入、受控runner/guard/工具manifest、两个native元收据和source-binding、历史剩余验收评估保留，早先评估的“未完成”是当时状态，不能当新CI或新运行。

实际FY/FX/FO 24命令/12次固定main、JADX21命令及JAR/WAR/独立driver在已归档`2026-10-17-recover-loop-transfer-finally-handler/results/final-runtime-v1`，当前产品30afa自身CI在同归档own-ci-v1；不将上述旧运行计作本包新命令。该包本身不完成父验收；随后父独审结论与最终任务状态见tasks.md及workflow-v1，剩余红项保持。

便携包独审325项/0错误，报告相邻保存；首次审计误用内嵌manifest大写名称而拒绝，v1报告与v2路径适配说明保留，未修改包或跳过检查。

本包完整EX raw也包含finThrow/nestedFin的fallback，它们是尚未实现的非覆盖型正向形，不能只登记EX.finReturn覆盖返回边界。已发现并恢复父实现待办；包无修改，原始完整方法输出可直接复核。

后续classfile定位memo不属于原63成员包。memo-v1仅历史report定位；memo-v2曾误把引擎class digest标为EX文件SHA，未接受该元数据；v3实读重算EX.class、两FinallyCompletion.class/目标Code及raw2/3/5的SHA并分别保存引擎标识，v2错误历史保留。物理Code/row定位仍不等于真实canonical CFG/SSA证明，后续必须实际probe。
