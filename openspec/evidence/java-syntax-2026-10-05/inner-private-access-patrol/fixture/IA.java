public class IA {
    private int secret = 42;                         // 外部类私有字段
    private int compute(int x){ return x * 2; }      // 外部类私有方法
    class Probe {                                     // 内部类访问外部私有（javac8 生成合成桥 access$）
        int readSecret(){ return secret; }            // 读私有字段（跨类）
        void writeSecret(int v){ secret = v; }        // 写私有字段（跨类）
        int useCompute(int x){ return compute(x); }   // 调私有方法（跨类）
    }
    int roundTrip(int v){ Probe p = new Probe(); p.writeSecret(v); return p.readSecret() + p.useCompute(3); }
    static long lflags = 0L;                          // long 位掩码（>32 位惯用法）
    static void setBit(int b){ lflags |= 1L << b; }   // 静态 long |=（恢复面）
    static boolean testBit(int b){ return (lflags & (1L << b)) != 0L; }
    public static void main(String[] a){ IA outer = new IA(); System.out.println(""+outer.roundTrip(50)+"/"+outer.new Probe().readSecret()); setBit(40); setBit(1); System.out.println(""+testBit(40)+"/"+testBit(2)+"/"+lflags); }
}
