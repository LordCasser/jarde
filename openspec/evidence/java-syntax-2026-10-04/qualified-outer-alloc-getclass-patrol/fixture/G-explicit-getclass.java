public class G {
    class In { int t(){return 1;} }
    // 用户显式写 o.getClass(); 语句（结果丢弃）——不得被折叠为 null-check
    In explicit(G o) { o.getClass(); return o.new In(); }
    // 对照：纯限定分配（getClass 是 javac 插入的 null-check）
    In plain(G o) { return o.new In(); }
}
