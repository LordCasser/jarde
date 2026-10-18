# 未修产品的19case完整控制基线

产品ee013/aabf CLI；root审两份完整Java并实际采集137命令274raw，原始4编译profile/8JVM152case同原。12Jarde API/default-all-essential全文一致、6方法Structured，raw方法map逐UTF8/物理owner/全BCI接受。Jarde24完整原样编译/48JVM912case同原，24wrong CodeSource拒绝；原4wrong root另计，driver-only classpath/freshparentless各case，目标构造器未运行。root独立review-v2实际exit0/无integrity findings，见extra/review.json。计数来自raw，捕获自身pending并非验收状态。

控制含String/Integer空/null/错误元素后的外层绑定与after值、raw与List<String>/Integer cast不匹配、typed-only正文，以及iterator/hasNext/next异常事件。原始控制源码合法，Jarde保持List<String>/Object头/Integer cast的mismatch可以整类编译运行。

JADX的javac23输入两份完整输出真实javac8/23均失败：错误将raw Iterator推为Iterator<String>，后续Integer cast成为不兼容类型。未裁剪生成源码，无JADX runtime。另两JADX启动被collector误用JDK8，真实major55/52工具失败保留；对应4次“无源文件”javac exit2是无有效源的工具失败，不算正常JADX源编译。另包Java23补跑这两格，见相邻supplement。完整源不因JADX失败而删mismatch方法。

root修正未执行Luna collector的pin-dict/bytes误用和Java文件名，root-v3再保留source_copies记录；未修改目标Java。独立checker Luna v1 compareSHA常量有字误，未执行；root-v2只修SHA并加强实际argv/CodeSource隔离核验后一次通过。3298来源/CLI/JDK/全部source/class/driver/工具前后与finally不变，target absent/5GiB/1GiB/defaultstack守卫接受。BLAKE3仅同次公开身份相等，不声称额外重算。
