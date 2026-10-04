public class TR implements AutoCloseable {
    private final String name;
    public TR(String n){ name = n; }
    public String use(){ return "used:" + name; }
    public void close(){ System.out.println("closed:" + name); }
    static String one(String n) {
        try (TR r = new TR(n)) { return r.use(); }     // 单一资源、无嵌套、无 widening
    }
    static String two(String n) {
        try (TR a = new TR(n+"a"); TR b = new TR(n+"b")) { return a.use() + b.use(); }  // 双资源
    }
    public static void main(String[] x){ System.out.println(one("p")); System.out.println(two("q")); }
}
