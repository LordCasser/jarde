public class IP {
    interface I { String go(); }
    static final StringBuilder EV = new StringBuilder();
    static String side() { EV.append("side-effect-ran"); return "v"; }
    // 前导声明带可观察副作用 + 末条直返匿名接口实例
    static I make() {
        String s = side();          // 前导 Declare，有副作用，且不被匿名体消费
        return new I() {
            public String go() { return "anon"; }
        };
    }
    public static void main(String[] a) {
        System.out.println(make().go());
        System.out.println(EV);
    }
}
