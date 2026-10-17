# Implementation and local validation

产品提交30afa7c9ab2818f41959f9ea6a8ca6f896714dc1。私有证明在1e7c2818c单独提交，再接入真实AST：正常Region七块与handler45单独owner闭合，For主锚13/header10/init8/update53；末poll后原子删除唯一fallback并派生handler四BCI与正常转移42/56。原异常边、公开Recovered/Region/AST形、通用named-catch拒绝保持。正常七块依靠既有Builder证明。有限大小hint之外没有新收费；满足35insn/3rows/8blocks大小hint的拒绝近邻可能先收费35steps，不能声称所有拒绝零账单变化。

- atomic-projection-native-v2：6命令均0，7项private/projection/evidence、2公库、1CLI，共10 passed/0 failed/0 ignored。其中1项原始探针，新增9 FQN。15个真实重新分析的拒绝class含正常null-stack进入handler。投影前Program dump保留For13与Fallback45，不冒充投影后AST；末IrItems拒付/真实取消由Eq断言保留原双树和usage。
- complete-local-native-v1：29命令均0，124次passed出现/0failed/8ignored；两filter重复4 FQN，实际120个通过测试。CI逐参数Clippy、reader人口1183/5279/545/3487/10、旧P5原golden/账单、finally/loop/typed-catch兼容、两API例、显式MSRV1.88、两版strict各364/0通过。被忽略的再生成测试不算通过。
- 4945行source前后稳定，实际Git full-tree NUL原始流和SHA1 blob/mode/native/CLI绑定在包内。默认stack、5GiB free/1GiB target，peak1016503496B，exact clean后target不存在。
- atomic与complete-local两个CLI别名都是60514880B、0555、SHA256 05e0272eb8cbc65acb87418cecda6d15571ddaea84c041d30436c1238e60b89d，属于同一产品字节。工具二进制未入包，收据保留其pins。

保留失败：first-projection-native-v1测试7pass，但build-cli错误使用bin jarde，整体failed/CLI None；atomic-native-v1重复Cargo参数，解析期失败，无产品测试；format-v1曾因生成测试lifetime拼写失败。修正后的新快照/收据另存，失败不计通过。

cache打包v1遗漏NUL原始流，不交付；v2补入完整37689610B Git raw；最终交付v3再补入早期失败批次的两份历史源码与重建说明，共146 payload +1 manifest成员。成员位于payload/，清单path相对payload；包内外MANIFEST逐字同，archive hash在独立summary。root重开tar对每项与原cache字节/hash回验。绝对路径仅为原始provenance，按上述路径重映射。这是证据包，非全仓离线构建环境。

本包仅实现/本地门；最终JAR/WAR/父样例/JADX与产品自身CI另列，不代替父计划验收。

早期first-projection源码闭合：build.rs为1925298B/SHA cb875bec4caac4202614ea87119ffaa672e25771df10a2883e86b95a465b1a06，report.rs为701348B/SHA16232705bd7feee7c529f4bd27a5066fc18085ca575c0ec65ef5c0f6680fff80，完整字节及重建操作说明入包，分别从可达1e7c2818c的build加三行私有模块声明、从atomic report去除较晚新增测试得到，均严格匹配该失败批次原始4945源manifest。其它4943行与atomic逐行同，不用较晚AST/证据测试冒充早期7pass。cache v1/v2与原失败收据保留；修正仅证据，不改30afa产品源码/CLI/CI。
