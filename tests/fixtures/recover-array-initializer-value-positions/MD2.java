public class MD2 {
    static int[] f;
    static int localPos(){ int[] x = new int[]{1,2}; return x[0]+x[1]; }   // 局部赋值位
    static int fieldPos(){ MD2.f = new int[]{3}; return MD2.f[0]; }        // 字段赋值位
    static int retPos(){ return new int[]{4}[0]; }                          // 返回位
    static int argPos(){ return sum(new int[]{5}); }                        // 实参位
    static int sum(int[] a){ return a[0]; }
    public static void main(String[] a){ System.out.println(""+localPos()+"/"+fieldPos()+"/"+retPos()+"/"+argPos()); }
}
