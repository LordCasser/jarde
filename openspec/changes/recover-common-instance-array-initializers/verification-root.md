# 实例数组共同前缀验收

当前产品基于6fd51a18静态片，只有report.rs/facade.rs修改。全部完整direct-super构造器的相同连续一维primitive数组前缀可提升为字段初值；super参数、每实例分配和不同suffix保留。this委托、不同实际参数、缺写/重复写、效果插入、参数依赖、逆字段顺序及handlers均保守保留。复用opaque AST、原field/init事实、现有emitter和member_texts，不建constructor graph或新pass。

root审private v6并补真实编译/发射断言修正。`results/scoped-rust-root-v7`实际Java层5/5、facade6/6；`validation-build-root-v3`按CI原allowlist的scoped all-feature Clippy、静态6/6及接口4/4回归、CLI构建成功。reader178/178及fingerprint5/5（1 ignored）实际成功，见reader-fingerprint-root-v1；守卫峰值904167534 bytes，未触发20GiB机器余量/1GiB target停止。

冻结CLI `/private/tmp/jarde-instance-array-cli-v1` SHA2565abb4bc82a50a8d427eff425ad80bc2fb845253b4216a81b60229e2337711663，metadata candidate-cli-v1.json SHAb86961182a352dfa663215c47bdd3e64cf7f8a67c5e36951f2c98d020f044cfa。root实际58命令/52cases候选回放完成：48 default/all渲染、4完整源码重编、6运行raw与各JDK冻结原程序一致。8反例32输出逐字等于旧CLI；只有3正例类5字段提升。root实际独立verify-instance-candidate-replay-root-v6退出0，接受361闭合文件、全部物理fields/methods、report.text/source_map及原owner/B3，结果instance-candidate-replay-independent-v6.json。来源仅核已接受baseline映射，handler旧goto@19缺口不冒称全BCI完整。v2/v4/v5验收脚本实际格式/变量错误与raw保留，v1/v3仅准备。

旧literal/ordered完整回归v3实际24命令/8腿全部重编-Xverify/raw通过，default/all正文相等，静态initializer proof完全保持、实例b正确提升；物理method.item、原report.text/source_map保持。root独立脚本接受97闭合文件，见static-array-regression-root-acceptance-v1.json。v2实际全部重编运行成功，仅错误比较包含request/usage的整个方法容器而误标失败，原manifest/raw保留。

当前tasks6/8。主线提交后的实例产品自己的完整双seed/JDK25/MSRV/fuzz/supply CI待验收，不能借6fd静态片CI或宣称EM18整单元完成。整数数组常量名私有v4尚未应用。

最终root-checkpoint-validation-v1：cargo fmt/all OpenSpec strict334/ git diff --check全部退出0，10产品/4test/16canonical当前pins均与冻结metadata一致。04:00 UTC只cargo clean本仓target，释放864.1MiB、target不存在，原source/class/raw及冻结CLI保留。
