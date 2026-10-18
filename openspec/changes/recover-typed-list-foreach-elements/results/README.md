# 原始 TestNestedLoops 与 typed foreach 实施入口

root已实际独立接受未修产品ee013/aabf CLI，当前实现尚未开始。OpenSpec任务1.1完成，其余六项未完成；不增加CF11或71/612整体完成数。

| 证据 | 实际范围 | 归档 |
| --- | --- | --- |
| original-baseline-root-v1 | 20命令40raw；完整原始outer+child debug/none，4原JVM/32case，8Jarde API，2JADX完整输出 | 77成员 / SHA256 823669b76a4ad3f63e42ae0d063a1c697ef88e970be860e8278ac063b5931a11 |
| generated-baseline-root-v1 | 75命令150raw；20原样编译/18成功，36 rebuilt JVM/288case同原，18wrong CodeSource拒绝 | 190成员 / SHA256 49c44b3fcf63375e0beec9f9f9e8c243b78dbbece2c84dae66256027990de446 |
| baseline-tools-root-v1 | 原始/矩阵collector与review各版本、root修正、源码pins、JADX/事务/外层绑定/规划的静态审查 | 40成员 / SHA256 6846f5860127f492a3da24ebfa6c777a55f7a3acb2a22205d81325bbbb576fa7 |

三包307成员已按相邻capture-review及embedded manifest逐字复开，见[归档审计](baseline-archives-root-review-v1.json)。仅观察和规划，不冒称新产品CI或新实现。

上游完整876B source SHA256 `4897fd18a15fce6e0abb40dc8afc6c7a3fa9f16e617f3c7e5e88072f6a37309e` 未裁剪。完整原源javac8遇IntegrationTest major55/expected52；javac23 --release8编译debug/none为major52。每profile完整2class，child ctor/test两Code、无字段，目标45BCI/119B，outer ctor/test两Code无字段。原始32case包含两CME、三NPE和正常/空列表后的实际列表状态，四profile/VM结果同，不硬编码oracle。parentless loader fresh每case、独立expected CodeSource、driver-only classpath；本Driver实际调用target构造器和实例test。

原始receipt SHA256 `4f48e3b14738cb4d3746763290788f525bba4bb1a1a3d47aebcf0111e6f8d482`；root审并实跑baseline audit root-v3接受。8API精确2方法Structured/无fallback、完整请求执行，两个List<String>形参但两个Object foreach/body String cast；所有物理BCI/UTF8/source owner都按每个raw RecoveryReport.text核，而非projected method/whole text。四组default/all在三个文本平面分别比较。BLAKE3只核same input身份/owner一致，未声称重算；输入物理SHA256另绑。

矩阵receipt SHA256 `46d5ff4f12d339d6fc2ddf9df963a256e25421b9f7c2b06eeec0cf5e08288012`。Jarde16完整选定child原样编译/32运行256case同原；JADX2完整outer+child编译/4运行32case同原，另2JADX javac8失败仍是原outer框架major55边界，无runtime。选定child完整源码编译不冒称整个outer源码编译；JADX保留outer完整源。20成功class/40Code均双parser独立重解析，完整class/method/field/access库存同原，物理指令向量/边界接受。未手修源、删方法或把原target混入runtime classpath。

最终matrix checker root-v4 SHA256 `9b434bb2a9badf72d11be77c0b642490b45b1c6f718495aa8d33a5595e6afbb2` 已实际执行接受。root-v3的真实driver tuple/JSON list比较拒绝保留；v4只规范JSON值表示，不改字节/顺序/Code。其他未执行草稿的schema/manifest/copy/负控契约错误及首次归档缺父目录错误在tools历史说明，不记作Java运行失败。3298来源/CLI/JDK/依赖/原源与全部生成source/class/driver前后及finally闭合不变，default stack/5GiB/target1GiB守卫接受，target不存在。

JADX实际两个String头与当前Object头构成呈现差距；不是旧iterator CFG拒绝，更不是当前语义错误。最终design保留合成header名和原body绑定，将精确受证cast吸收入typed头，不新增作用域solver。原名s1/s2头折叠、冗余iterator声明、equals/remove Object包装仍独立，不能声称此片将通过整条原上游文本黄金。
