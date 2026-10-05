public class MF {
    transient int t = 1;                       // transient 字段
    volatile int v = 2;                        // volatile 字段
    static final transient int SF = 3;         // 组合修饰符
    native int nat(int x);                     // native 方法（无 Code——像 abstract）
    strictfp double fp(double x){ return x * 2; } // strictfp 方法
    synchronized static int ss(){ return 4; }  // 同步 static（ACC_SYNCHRONIZED）
    final int fin(){ return 5; }               // final 方法
    private static synchronized void psync(){} // private+static+synchronized 组合
    public static void main(String[] a){ MF m = new MF(); System.out.println(""+m.t+"/"+m.v+"/"+SF+"/"+m.fp(2.0)+"/"+ss()+"/"+m.fin()); }
}
