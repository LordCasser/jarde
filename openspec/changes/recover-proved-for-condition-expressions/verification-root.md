# 当前验收断点

2026-10-12，7/9。1.1固定真实基线/同IR首拒已接受，1.2真实控制基线接受，2.1最小生产match已root逐字应用。永久fixture/API、候选完整对照及公开停止证据已接受；最终CLI与fresh workspace已接受，自身CI和clean Git交付仍待，不能将静态candidate或前产品CI当当前验收。

控制实际57commands/114raw，原双JDK19行oracle相同；JADX完整生成三类四腿同原。Jarde两个正例各四隔离完整类+其它原类腿共八成功同原，三生成类合并四失败、拒绝类独立四失败均no runtime。LeadingTickAndBreak既有Fallback/ExplanationOnly，source-map缺physical BCI [1,2,3,7,8,11,18,19,20,21,24,28] 四profile同，独立债务，不扩大本片去修自然环或fallback；其它所有方法physical BCI/UTF8/身份/Code/flags及default-all一致。BLAKE3 digest仅shape/owner/location一致核验，物理文件SHA256独立核，不声称BLAKE3独立重算。

详见results/controls-baseline-root-v1/acceptance.json。Collector首次preflight局部import re的NameError、checker错把BLAKE3当SHA256的失败，以及随后忠实拒绝fallback source-map缺口的失败均保留。root corrections分版保留，没有改生成Java消除失败。

新增fixture采用root真实javac23 --release8编译字节，source/byteSHA与两JDK字节差异在tests/fixtures/java8/cf11-for-condition/README.md。真实javap增5class/24Code/0handlers/41branch/0jsr；Reader只更新测得的人口注释和tuple，生产Reader未改。Corpus manifest实际再生成通过，Reader census和正常fingerprint sweep均通过。

主线HEAD593fbcc0b仍是上一片关闭记录，当前有未提交生产/test-fixture/docs改动。资源5GiB free/target1GiB一秒守卫，fresh workspace v4与最终构建均实际完成clean，target不存在。不新建branch/worktree，整个CF11与持续目标尚未完成。

候选v3定向实际15命令（含clean），12次测试汇总422/0/2，含corpus再生成和正常sweep的重复目标，不当唯一workspace总数。11个独立Cargo测试目标通过，未调高RUST_MIN_STACK；target峰值710170385bytes并clean。CLI SHA51c9a1326031fd10444f7e18a055c6daecef233423c90478257becf24d1e0bd4/61213840bytes/0555，4718source inputs前后恒同（有意manifest再生成另记），见results/focused-build-root-v3。

候选完整三组postfix实际115命令/230raw，28个完整生成compile/run成功腿；所有11个control loops各profile一个for，固定outer一个for/inner一个while。全部正例physical来源完整、default/all一致，非目标方法text/map/quality逐个与旧基线相同；controls8个既有编译失败/no runtime及fallback缺12BCI精确相同。numeric十五项（含NaN/null/越界/long极值）和固定九项oracle一致。root acceptor v1历史fixed label不同导致KeyError未接受，v2只适配schema后通过，失败保留。详results/postfix-candidate-root-v1。

固定下一项TestSequentialLoops2实际29命令/58raw基线已保存于cf11-testsequentialloops2-baseline-root-v1，原/JADX十行一致；Jarde正确恢复两循环和char[]clinit，但return尾部new/dup/constructor条件值未恢复，四combined+四同source isolated compile失败/no runtime，map缺[88,94,95,96]。内部首拒尚待同IR实测，此独立债务不混入当前条件for。

公开late Stop实际通过（results/public-budget-root-v1）：按测试动态limit+1推得source2631，limit/observed AnalysisSteps2630，sumWithChangingFieldBound()I@BCI1，physical method identity见raw；未产生body text/map/content，早先完成成员保留不称全类没有产物。pre-cancel全部工作usage零、无declaration/fields/methods/text，只elapsed_millis为墙钟排除；非异步取消。nested5/0/0、正常corpus5/0/1、4718pins恒同，target peak354339918B并clean。

2026-10-12：完整回归发现 F1/F2 及 Unicode ASCII 控制仍冻结旧 while 形态。仅在两个既有测试中对两个准确数组长度/+2块做一次局部投影，历史两份 F1 golden 未改，其余完整类全文与其他 ASCII 类仍逐字核。F1 原 class/生成完整类运行比较、F2 try/catch 和旧14输出通过，专项5/0/0；Unicode 专项4/0/0，已有中文标识符完整类编译运行对照通过。workspace v1 删除 examples 路径守卫失败，v2 在 command32 陈旧 F1 断言失败，v3 在 command75 Unicode 陈旧 F1 断言失败，均结束 clean/pins恒同；不是全仓通过，前缀批次不拼接验收。原始失败和私有候选/实际应用分别保留于 results/regression-repair-root-v1、workspace-failed-root-v3、unicode-regression-root-v1；v4重新从 target 不存在开始完整95批/360目标运行，待实际结束验收。

2026-10-12：最终本地验收7/9。fresh workspace v4从零运行95批/360目标恰好一次，3461/0/97；root独立从196raw重建3558个named outcomes，98命令退出/guard与4718项before/after/live闭包恒同，target峰814348361B并clean，见results/final-workspace-root-v4。CI同范围Clippy/fmt/global strict352/0和最终CLI构建通过，target峰432717899B并clean，见results/final-lint-build-root-v3。最终CLI61213840B/0555/SHA51c9a1326031fd10444f7e18a055c6daecef233423c90478257becf24d1e0bd4与115命令已实测candidate完全相同；root实际核最终source closure与workspace同版本，变化仅三份测试源码和fixture README，production inputs恒同。据此接受28个完整生成runtime成功腿的同字节绑定，明确没有在final路径重跑115命令；8个既有Refusals编译失败/no runtime保留。自身product commit/run与clean Git尚未接受。
