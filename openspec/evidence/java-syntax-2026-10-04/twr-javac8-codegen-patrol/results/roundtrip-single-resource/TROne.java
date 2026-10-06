public class TROne implements AutoCloseable {
    private final String name;
    public TROne(String n){ name = n; }
    public String use(){ return "used:" + name; }
    public void close(){ System.out.println("closed:" + name); }
    static String one(String n) {
        try (TROne r = new TROne(n)) { return r.use(); }
    }
    public static void main(String[] x){ System.out.println(one("p")); }
}
