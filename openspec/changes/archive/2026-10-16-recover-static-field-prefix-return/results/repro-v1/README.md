# 静态前置返回的当前基线

本包只移交规划基线，未实现静态前置更新。64条native raw来自两套实际实验：finally screen 26命令（两原编译+12原Java8/23运行+6CLI均0，六份完整生成源重编均1，无rebuilt运行）；新增独立Probe 6命令（两driver编译、四次原FO调用均0、60B六行逐字一致）。源码/原class/生成Java/argv/退出码/raw/工具与CLI pin均保留，归档已逐成员回读。

FO失败来自bump()I的静态前置新值copy不被恢复；evalThenFin已生成正确顺序但未获得完整类重建运行证据。FY/FX的独立finally拒绝不能归本片关闭。所有原Java、完整生成文本与driver未修改；没有删方法使编译通过。

基线产品为ba3fa76af，冻结CLI c44bde2c…fa396，4907输入。该提交自身CI因漏列新EM11 fixture指纹失败，不能当成功产品门；前片必须先补维护与新提交CI再收尾。实际后续实施基线须重绑定。JDK/CLI/Python二进制外置，不包含在包内。固定driver验证不证明任意程序语义等价。
