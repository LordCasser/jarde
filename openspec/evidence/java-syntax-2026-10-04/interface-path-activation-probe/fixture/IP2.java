public class IP2 {
    static final StringBuilder EV = new StringBuilder();
    static String side() { EV.append("side-effect-ran"); return "v"; }
    // 顶层接口 + 无参 + 返回恰为 ()LTopI; + 前导 Declare 带可观察副作用 + 末条直返
    static TopI make() {
        String s = side();
        return new TopI() {
            public String go() { return "anon"; }
        };
    }
    public static void main(String[] a) {
        System.out.println(make().go());
        System.out.println(EV);
    }
}
