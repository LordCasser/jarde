# 首次实现 root 验收记录

2026-09-23，CLI SHA-256 `feed5c377a439490efb12a2a46bdfa0e94c32dffd3a83bd1583b70fe89317350`。本轮没有运行Cargo或重建CLI，使用独立新工作目录执行原class/JADX/jarde完整类。

- `deferred-evaluation/root-acceptance/`：五份独立审计3+18+24+12+12项共69项，原class/JADX/jarde逐行一致；jarde零引用、完整javac成功。
- `root-acceptance/permanent/`：精确重编译并重新应用14份Code补丁，1338B/19Code/hash与冻结fixture相同；83项原class/JADX/jarde逐行一致，jarde零引用和完整javac成功。与上述场景有重叠，不宣称152个不同语义类别。
- `deferred-evaluation/composite-boundary/`：新增15项中11项仍错，原class/JADX一致；jarde零引用且完整javac成功。支持producer的直接reader进入iadd/ineg后，最终return的求值位置未被当前证明覆盖。
- `negative-review/analysis.md`：另外记录源码级scope、未知消费、深度、命名预算风险及root审读限制；尚未执行的风险不能冒充运行证据，也不要求建立新作用域框架。

结论：部分正面闭合已经成立，整体未验收。2.1、2.5与3.1退回；1.2、3.2、3.3仍未完成。生产/Cargo窗口重新交原实现Luna补齐，root继续准备独立反例和复核。Java包/adjacent/Cargo门禁的worker报告不能代替本轮尚未进行的root最终门禁；语料新增lambda/type-qualifier/primitive-conversions仍待统一冻结。

后续root独立补查仍使用同一feed5c：`inline-controls/before/`普通javac的60项原/JADX/jarde全相同、零引用，用于约束不能倒退原本正确的内联。`negative-review/switch-core/`为去掉无关keepPool后重新编译/合法修改的独立输入，6项原/JADX相同，jarde整类零引用且编译成功，但正常两项重复producer（trace12→121、32→323）。switch边界的前置返回呈现与arm呈现尚未共享一次求值决定，task1.2须正常正确呈现或来源完整拒绝，不强制新增跨区域提升机制。

## ecab 修订版 root 验收

CLI `ecab8244d1709765fa0f2b0effcebe49b9f066eea911843d11d34abec5837330`，固定副本 `/tmp/jarde-cli-deferred-final-ecab`。在全新临时目录独立重编译输入、精确patch并重新执行，未覆盖先前失败基线：

- `root-final-ecab/` 的69项、其 `permanent/` 的83项、`composite-boundary/root-final-ecab/` 的15项，以及 `inline-controls/root-final-ecab/` 的60项均原/JADX/jarde逐行一致，零引用、完整javac成功。各组有语义重叠，不把数量相加宣称不同类别。
- `negative-review/switch-core/root-final-ecab/` 仍未闭合：producer quote从零变为2，但预生成return仍写可执行调用；整类javac成功，6项全部错序。正常12→21/32→23，生产者抛错1→21/3→23，marker抛错12→2/32→2。拒绝必须约束消费者，不能只在producer位置加入注释。
- `nested-producer-boundary/` 新增合法直线反例5项，SHA `f77378eebaa6230967194214ca0ef2ffd3e1a3f9235e3ff92e567b9c98f8677f`；原/JADX一致，jarde零quote、整类javac成功但5项全错。`value@0; mark@3; take@6; mark@9; ireturn@12` 被改写为先mark、再保存take(value())。candidate_producers直接父子过滤遗漏了inner→outer之间的独立效应；不能无条件由外层声明覆盖内层。
- `negative-review/failed-declaration/` 的worker实测6项原/JADX一致，ecab保留shift/local声明/call/return拒绝来源，没有未声明saved名字；jarde完整javac失败，不声称已恢复执行。root已在其root-ecab子目录独立重放并断言6行、同一hash及所有相关来源，结果相同。

整体仍未验收，task2.1/2.5/3.1和最终门禁保持未完成，生产/Cargo窗口再次交原worker修订。4组独立后续fixture现已由root固定，98class/639Code/81handler/236targets/8subroutines，251个fingerprint输入，原232条未变；这不是本实现通过证明。

补充普通同步14项 `guard-inline-core/` 在feed5c/ecab均原/JADX/jarde一致、零引用、全类编译成功；后续修订须保持。较大的带资源21项输入存在独立guard拒绝与JADX重复close，已单列，不扩大本修复。
