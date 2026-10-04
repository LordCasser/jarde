public class IP3 {
    // 纯直返形（无前导声明）—— 已验收能力，writer 应能复现 root
    static TopI make() {
        return new TopI() {
            public String go() { return "anon"; }
        };
    }
    public static void main(String[] a) { System.out.println(make().go()); }
}
