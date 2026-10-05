public class FB {
    static class B {
        int a; String s;
        B a(int v){ this.a = v; return this; }      // this 返回链方法
        B s(String v){ this.s = v; return this; }
        String build(){ return a + ":" + s; }
    }
    static String fluent(){ return new B().a(1).s("x").build(); }        // 纯链
    static String broken(){ B b = new B(); b.a(2); b.s("y"); return b.build(); }  // 拆语句（对照）
    static String mixed(){ return new B().a(3).build(); }                // 短链
    static B reuse(){ B b = new B().a(4); return b.s("z"); }             // 半链半引用
    public static void main(String[] args){ System.out.println(""+fluent()+"/"+broken()+"/"+mixed()+"/"+reuse().build()); }
}
