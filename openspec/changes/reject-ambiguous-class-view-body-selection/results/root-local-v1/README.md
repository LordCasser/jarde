# class-view 根本地对照与门禁

本归档保持旧实现与修复候选各自真实源码和日志，不改历史失败。manifest逐项记录唯一相对路径、字节数与SHA；根已重新打开全部成员逐字核对。

旧生产为exact d3/main7d0：library 首abstract-code组合的Name已按预期歧义，首listed-ID随后错误执行（0通过/1失败），后八组合与第二ID未跑。CLI v1先因测试错读顶层usage得到Null/0，**排除产品反例**；仅将新测试全部usage修成实际execution.usage后，fresh CLI v2真实列举成功，首listed-ID实际exit0、期望歧义exit3（0/1），后续ID/Name未执行。两次receipt各4900来源前后恒同、guard未触发、finally exact clean成功。

修复保留foreign-owner优先级，Method与Name共用现有完整候选判定，移除首取helper。focused-v1实际76通过/0失败/2忽略，5suites、8命令/16raw，root4952checks接受。成功公共矩阵到达九组×Name+两个listed-ID共27请求；abstract-code三个请求均method_bodies=0，仍歧义且零prep/body/Code/IR。唯一abstract/native/Code、不同descriptor、同签名foreign-owner、missing/partial/stopped-body和一次读取/按需解码控制全部通过。CLI真实list→两ID及Name三请求均exit3、完整index/flags/相同IDs保留，execution.usage零body/Code/IR。

fmt、当前CI Clippy、OpenSpec1.11.0/1.14.1 strict366/0、实际build CLI及finally exact clean成功。冻结CLI59599048字节/SHA c0bc6b64e1c763ee400afdb60d328a83151091fecbd72ed8ead1e906bd170f45，不包含可执行文件本身。4900来源从本轮运行开始包含全部manifest corpus遍历输入，区别于上轮4887历史闭包的13项audit-time补充。本地focused不替代产品自身完整CI，交付2.2仍开放；另外两项proof-stop新计划仍0/23。
