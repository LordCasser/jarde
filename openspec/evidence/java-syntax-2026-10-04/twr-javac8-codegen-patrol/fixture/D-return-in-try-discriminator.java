public class D implements AutoCloseable {
    private final String n;
    public D(String s){ n=s; }
    public String use(){ return "used:"+n; }
    public void close(){}
    static String noReturn(String s){           // A: 无 return-in-try（TwrAudit 同形）
        try (D r = new D(s)) { System.out.println(r.use()); }
        return "done";
    }
    static String withReturn(String s){         // B: return-in-try（我的 TR.one 同形）
        try (D r = new D(s)) { return r.use(); }
    }
    public static void main(String[] x){ System.out.println(noReturn("p")); System.out.println(withReturn("q")); }
}
