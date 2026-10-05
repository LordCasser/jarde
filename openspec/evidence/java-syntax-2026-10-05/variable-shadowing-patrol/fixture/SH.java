public class SH {
    private int v = 1;
    private static int s = 2;
    int readField(){ return v; }                          // 直读字段
    int shadowLocal(int v){ v = v + 10; return v + this.v; }  // 参数遮蔽字段（this.v 区分）
    int shadowBlock(){ int x = 1; { int x2 = 0; x = x + 1; } int r = x; { int x3 = 5; r += x3; } return r + v; } // 块作用域并行局部
    static int shadowStatic(int s){ s = s * 2; return s + SH.s; }  // 参数遮蔽静态字段
    int innerShadow(){ final int v = 100; Runnable r = new Runnable(){ public void run(){ System.out.println(v + SH.this.v); } }; r.run(); return v; } // 内部类捕获遮蔽名
    public static void main(String[] a){ SH s = new SH(); System.out.println(""+s.readField()+"/"+s.shadowLocal(5)+"/"+s.shadowBlock()+"/"+shadowStatic(3)+"/"+s.innerShadow()); }
}
