# Root验收：嵌套int数组复合更新

产品、完整Java语义对照及最终Rust边界/停止、指纹/P5已经通过；当前任务7/7。确切产品CI的完整门禁仍待验收，不能把本片或DT26整个单元记为完成。

## 根因与处理范围

P02_multianewarray的capture、SAM适配和原sum/main本来已恢复；拒绝发生在lambda helper的`aaload; dup2; iaload; ...; iadd; iastore`。verifier保守地把行引用记为Unknown，dup2的写入副本也保留Unknown，旧prove_array_update提前要求该副本有int[]类型，因此在既有证明能够处理之前退出。

本片只删除该提前gate。后面的原始行值仍必须由既有array_of_value沿aaload准确降rank证明为int[]，四个不同的dup2输出必须有准确读写关系及唯一消费，原有左值前缀/RHS依赖、顺序、深度和预算停止检查均保留。不新增pass、AST、全局复制类型推断或类型服务，也不扩展其它操作符。

JADX本地源码v1.5.6-26-g2fb1b163用SimplifyVisitor/PrepareForCodeGen与InsnGen处理这些形态；实际对照执行的是安装版1.5.6，两者身份单独记录。JADX完整输出采用普通数组赋值或保存行引用。本片复用jarde已有compound证明，不复制第三方代码；有副作用的左值只能按原身份和次数求值。

## 实际对照结果

- 原P02两真实编译器输入的旧完整候选均compile1，helper fallback；fresh JADX default/none四腿完整源码隔离重编及运行4/4，root独立78checks。见BigDecimal results/nested-array-jadx-baseline-v2与nested-array-jadx-baseline-root-verification-v2.json。
- 新NestedIntUpdates原source在真实Corretto8/javac23编译并javap和运行，冻结十行oracle，覆盖二维/三维/scalar、行/index/RHS trace123、outer-null/OOB、null-row/inner-OOB、RHS换行对象和不同左值。旧jarde两个完整类compile1；fresh JADX四腿完整类加同一外部Runner重编执行4/4，root独立91checks。见results/controls-v1、controls-baseline-v1和controls-baseline-root-verification-v1.json。
- 冻结新CLI`/private/tmp/jarde-nested-int-array-cli-v1` SHA256 `8745071312981d4eafb8fbb738e6a1fa1edbb55753062bfae057e879c494bb7a`，五产品源及最终五测试/工作流身份见results/candidate-cli-v1.json。fresh candidate-v2实际12commands/4完整腿全部通过。各腿用自己的真实JDK、空classpath/sourcepath编译全部目标source，Runner为固定外部oracle，运行仅新classes并-Xverify:all；exit/raw stdout/stderr均与原流一致。P02原ctor/sum/main正文保持，helper全部11个物理BCI保留。root独立verify-candidate-v6.py **474checks/0errors**。
- 旧24完整控制腿使用同一新CLI重新render/编译/执行，**24/24**；原BigDecimal独立verifier对新输出检查**1226checks/0errors**，102commands/384closedfiles。见results/legacy24-candidate-v1与legacy24-root-verification-v1.json。该回放及新4腿原oracle/JADX流逐hash复用已冻结记录，不能称本次fresh原程序/JADX执行。
- NestedIntBoundaries两真实JDK完整编译/javap；返回复合更新的dup_x2额外消费、merged行Phi均不冒称本片+=，物理成员和原始范围保持。新CLI两份完整报告由root核验**64checks/0errors**，见results/negative-original-v1、negative-candidate-v1、negative-root-verification-v1.json。它们只用于负例来源验收，没有runtime成功声明。

## 本地门禁与仍待执行项

最终负例patch之前，focused旧capture/新正例7ordinary通过，双JDKignored完整语义比较均通过；与BigDecimal旧断言修正一起跑的focused12ordinary通过/2ignored。325 Java-lib通过；reader真实新census测试1项通过，精确为(1071,4654,463,2715,8)，相较前片新增4classes/28Code bodies/4branches，生产reader没有修改。早期日志在BigDecimal results/root-nested-*，新结果在本片results/root-*；root-focused-boundaries-v1名称早于最终patch应用，不能当作两个新增边界测试已跑。

最终patch包含负例和positive类的public budget/cancel测试，但root-final-boundaries-v1因空闲低于20GiB未启动，故尚未通过Rust编译执行。该测试只覆盖公开class-source的晚期预算和预取消，不声称内部builder checkpoint压力验证。fingerprint新增11个准确输入，2029旧row和分类不变；root用临时Python blake3 1.0.8更新并逐项核对，官方Rust指纹校验与P5严格pins仍待CI。未修改P5阈值。

fmt、git diff check和OpenSpec all strict **330/330**通过，原始双流见results/root-readonly-gates-v1.json。最终MSRV、CI-exact Clippy、新Rust边界及双seed全workspace尚未验收。只清当前jarde target释放701.7MiB，冻结CLI和原始失败不删；其它项目占用使清后仍低于20GiB，不启动本地Cargo编译。失败patch/v1回放路径/v3 verifier字段错误保留，并由v2 patch/v2回放/root v4独立校验修正，见results/validation-failures-root-v1.json。

## CI修复与后续验收

BigDecimal旧产品6997c9f8的CI37989319644真实命中过时closedNumberBoundary预期；修正提交8cd8c4f的CI37991578328第一轮workspace已通过，第二轮仍在运行。该run的supply job在构建cargo-deny Docker action时连续Docker Hub429，尚未checkout或执行advisory检查；原REST日志在BigDecimal results/ci-supply-build-failure-v3，不能称为依赖政策拒绝。

工作流改用固定cargo-deny0.20.2的官方独立CLI安装，保留root/fuzz的all-features/workspace/locked/config四项检查，不放宽政策或改lockfile。root本地同版本对两个workspace实际均advisories/bans/licenses/sources ok。见results/root-deny-workspace-v1与root-deny-fuzz-v1；官方安装资料为https://embarkstudios.github.io/cargo-deny/cli/index.html。

本片产品推送后必须绑定完整commit SHA收取确切CI原JSON和稳定任务日志，核验四job、双seed、真JDK25及新增nested ignored步骤。若旧8cd run仅供应链失败，可以仅重跑其失败任务以单独完成BigDecimal验收；它的成功不能替代本片新产品CI。未达到这些条件前保留任务2.2/3.1/3.2/3.3未勾。

提交前完整cached diff检查因原始stdout/stderr和正式diff补丁的空白字节返回2，这些字节绑定已记录hash，保留原样。将这些准确证据路径列为例外后的681项源码/spec/script/JSON检查返回0，不修改全局Git属性；见results/root-staged-diff-check-v1.json。v4独立443checks已过，v6增加实际命令退出与物理更新/副作用来源、次数检查后474checks通过；v5因继承旧输出文件名拒绝覆盖，旧v4结果保留。

## 2026-10-10 最终本地补验 — 任务5/7

产品561de209531c021a9d8adb7c979bae5a57d6c3fb已经推送main，[确切CI37994276707](https://github.com/LordCasser/jarde/actions/runs/37994276707)运行中。空闲随后回升到27GiB，root-final-boundaries-v2实际14ordinary通过/2ignored，包含两个新增负例和public late-budget/pre-cancel测试；不能继续把这些项目写成未执行。官方Rust指纹5pass/1ignored验证了最小11条增加与分类，P5严格pins5pass/1ignored通过且未改pins。五产品/最终五测试源仍与冻结metadata一致，见results/local-root-acceptance-v1.json。2.2/3.1完成，3.2/3.3仍待确切CI全门禁；本地没有重复fresh双seed全workspace命令。

8cd8c4f旧CI37991578328已完整结束：stable/MSRV/fuzz成功，唯一supply失败仍是DockerHub429。原完整JSON ci-run-v2.json及stable raw日志无损gzip ci-stable-job-v2.log.gz已落盘；raw1168629bytes/SHA9edd88a86d2602c118560498540fe97d1e0a6a5890df5454f9fa7f6aaf7b59e5。root已仅重跑失败任务，不重跑成功stable或把其证据借给nested产品。

## 确切组合产品CI与最终验收

561de209531c021a9d8adb7c979bae5a57d6c3fb的CI37994276707完整终态success，四job/51steps全部成功。ci-run-v1.json与完整ci-stable-job-v1.log.gz已保存；stable原始1172292bytes，SHA9b70099c24c932aaea3616ad5a6a6d82d891e1894ad753ab25e72d5eb26f7160，未trim或改写。root实际运行verify-ci-product-v1.py，ci-product-root-acceptance-v1.json接受准确commit/CLI/product与final test/workflow/canonical身份。

两个fresh全workspace/all-targets/all-features/locked固定seed各354test-result记录、3348passed/0failed/96ignored，三个新ordinary结构/负例/停止测试在两轮均成功。真Temurin25.0.4+7.0.LTS安装路径及日志已核对；BigDecimal显式ignored2pass、nested全类显式ignored1pass，其它必要Java/MSRV/Clippy/fuzz/OpenSpec及依赖政策步骤均成功。新独立cargo-deny0.20.2的root/fuzz四政策各ok，没有借旧Docker失败run。任务3.2/3.3完成，本片7/7，DT26/71整单元分类仍不变。

低空间时root只清本仓421MiB缓存，冻结新CLI和raw证据保留；随后机器空闲恢复25GiB，按实时20GiB守卫推进下一片。新returned OpenSpec规划4/4、基线/架构2/7，尚无实现验收。
