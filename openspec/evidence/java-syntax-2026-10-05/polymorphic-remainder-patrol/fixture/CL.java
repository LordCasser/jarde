public class CL {
    static class P { int f(){ return 1; } }
    static class Q extends P { int f(){ return 2; } }
    static class R extends P { int f(){ return 3; } }
    static int crossCall(P p){ return p.f(); }                    // 多态参数
    static P choose(boolean c){ return c ? new Q() : new R(); }   // 异构三元返回上转型（三元汇合域）
    static java.util.List<P> poly(java.util.List<P> in){ return in; } // 泛型直传
    public static void main(String[] a){ System.out.println(""+crossCall(new Q())+"/"+choose(true).f()+"/"+poly(java.util.Arrays.asList(new P())).size()); }
}
