public class OP2 {
    static int shl(int x){ x <<= 2; return x; }            // 移位复合 <<
    static int shr(int x){ x >>= 1; return x; }            // >>
    static int ushr(int x){ x >>>= 1; return x; }          // >>>
    static long lshl(long x){ x <<= 3; return x; }         // long 形
    static float nanf(){ return Float.NaN; }               // NaN 常量
    static double pinf(){ return Double.POSITIVE_INFINITY; }
    static double ninf(){ return Double.NEGATIVE_INFINITY; }
    static float pzero(){ return -0.0f; }                  // 负零
    static boolean condAssign(int x){ return (x = x + 1) > 0; }   // 条件内赋值（返回新值）
    static int condAssignOld(int x){ boolean b = (x += 1) > 0 && x > 0; return b ? x : -1; } // 复合+多读
    public static void main(String[] a){ System.out.println(""+shl(3)+"/"+shr(-8)+"/"+ushr(-8)+"/"+lshl(2L)+"/"+nanf()+"/"+pinf()+"/"+ninf()+"/"+(pzero()==0.0f)+"/"+condAssign(0)+"/"+condAssignOld(0)); }
}
