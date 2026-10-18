# Bulk执行区间独立修复证据

本片与typed foreach规格/实现分开。实际失败是b78候选第一fresh seed第三测试batch/command4的bulk_recovery_lifecycle重叠门：gate和worker的两个实际重叠断言通过，公开峰值却1。root核全源pins不变/真实raw及exactclean；不是Java输出差异，也不是成功完整seed。

- native-failure-root-v1：19成员，SHA256 ada99433db8ca7370b9adfaa03b0c7900ac206e4e5fa13e01cbf9267fa4f0b66；完整失败收据/raw/metadata及root复核、Luna最小patch。
- runner-contract-failure-root-v3：8成员，SHA256 eea8ba74e156e07a1ab0eab89d310aa1484c3548382062813a11ae692126cf41。修正后4 bulk目标16测试全部通过，但runner清理helper只接受每命令一个test binary，遇4个binary拒绝；源不变且finally exactclean。此为采集/收尾契约失败，不改写成广门通过或语义失败。

v4已将4 bulk目标拆成各自命令，额外4次精确真实重叠回放，随后fmt/CI Clippy/MSRV/Java/Reader/P5/CLI。当前进行中，无新完整native/产品自身CI接受。唯一生产差异是src/bulk.rs原_entered guard移到共用run_class_task入口、panics_at/gate/delay之前，加一条注释；无其它计数器、锁、线程、机制或旧阈值修改。源与私有Lunapatch SHA见原失败归档extra。旧b78 CI39111223906仅历史候选，不代表修复最终提交自身CI。

v4真实定向部分已通过：单独4个bulk目标4+7+3+2=16测试，4次精确重叠回放各1通过；全部旧断言/timeout不变，20通过/0失败/0忽略。root核首入口guard/原Drop与gate不持registry锁，task1.2实际验收勾选；完整广门尚在进行。

最终v4广门已root独立接受：26命令含clean，822通过/0失败/6旧忽略，fmt/CI Clippy/MSRV/Java496/Reader181/全部P5与公开门通过；3190来源未变，fingerprint2429未变，默认stack/5GiB/target1GiB守卫/exactclean通过。CLI63405424B / SHA256 85ca6e7d36c0edeb72f7f6a3c33c405f203028fb21d128522f0b0a3a92e1cf78，broad-final-root-v4为56成员/a5237ee1559dbcaf03d6d15b662c4cd28c1fb3cf671e2bd2d31e9ab078a1f8d3。

最终20同argv API/40raw已独立review接受，除usage外全部JSON、whole/projected/raw文本/maps与原完整Java矩阵对应d130逐字一致，3301来源/所有冻结物理产物等pins前后闭合。final-api-replay-root-v2为85成员/5377072d611ee7f1b3cf6b874f4428515ec85651bf5a91b18c9c02517bd46898；本次未重跑JDK，精确相同完整Java复用typed原始final36JVM/288case及控制Jarde48JVM/912case，JADX真实编译失败保持原样。

本片3/4、typed5/7，未验两fresh native/最终提交自身CI。prepared implementation-acceptance-tools-root-v1为20成员/6a449ec758872a123e8d23b7ecfff03c322458bd8f8863be9c4c6959268527e7，包含实际Luna最小patch、ownCI v3（12源码/3fixture/54门）、native wrapper v3（9门/385目标）及精确base checker、产品Git binder；工具准备不是实际native/CI验收。原名fold仅静态备忘，独立未实施。

最终e69两fresh native已root独立接受：每轮385目标/101批/3759通过/0失败/98忽略，9精确关键门每轮各一次；额外全部3857具名(name,status)逐target相等。来源各5143/构建3190并集5150按Git blob/bytes/mode绑定e69，CLI85ca不变、target absent。workspace-seed1-root-v2（213成员/255c6b24…）、workspace-seed2-root-v2（212/7f1dfbee…）、workspace-acceptance-root-v2（22/23731b67…）含实际收据及checker/binder/具名review；不是仅工具准备。superseded-ci-root-v1（6/6dcaf040…）保留取消旧候选的原始动作，不算最终自身CI成功。bulk3/4、typed6/7，余实际e69自身CI/strict/干净main闭合。
