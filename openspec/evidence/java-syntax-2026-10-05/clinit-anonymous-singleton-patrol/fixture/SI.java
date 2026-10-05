public class SI {
    interface Op { int apply(int x); }
    static final Op DOUBLE = new Op(){ public int apply(int x){ return x * 2; } };       // clinit 匿名（无捕获）
    static final Op ADDONE;                                                              // blank final——clinit 赋值
    static final java.util.Map<String,Op> REG = new java.util.HashMap<>();               // 注册表模式
    static {
        ADDONE = new Op(){ public int apply(int x){ return x + 1; } };                   // static 块内匿名
        REG.put("d", DOUBLE); REG.put("a", ADDONE);
    }
    static final Runnable CAP = makeCap(7);                                              // 经方法构造的匿名（工厂）
    static Runnable makeCap(final int n){ return new Runnable(){ public void run(){ System.out.println("cap:" + n); } }; }  // 捕获匿名
    static int run(String k, int v){ return REG.get(k).apply(v); }
    public static void main(String[] a){ CAP.run(); System.out.println(""+run("d", 5)+"/"+run("a", 5)); }
}
