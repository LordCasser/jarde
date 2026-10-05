public class NP {
    static long ternaryPromo(boolean c){ return c ? 1 : 2L; }                // int 分支提升到 long（iconst_1; i2l）
    static double ternaryDouble(boolean c){ return c ? 1 : 2.5; }            // int → double（i2d）
    static byte compoundNarrow(){ byte b = 10; b += 5; return b; }           // 复合赋值隐式收窄（iadd; i2b）
    static byte compoundNarrow2(){ byte b = 100; b += 100; return b; }       // 溢出回绕（200 → -56）
    static char charCompound(){ char c = 'a'; c += 2; return c; }            // char 复合（iadd; i2c）
    static int shiftMask(int x){ return x << 33; }                           // 移位距离掩码（33&31=1——字节码常量折叠）
    static int shiftByLong(int x, long d){ return x << d; }                  // long 移位距离（l2i 后 ishl）
    static int shiftByLongConst(int x){ long d = 33L; return x << d; }       // 常量 long 距离
    public static void main(String[] a){ System.out.println(""+ternaryPromo(false)+"/"+ternaryPromo(true)+"/"+ternaryDouble(false)+"/"+compoundNarrow()+"/"+compoundNarrow2()+"/"+charCompound()+"/"+shiftMask(8)+"/"+shiftByLong(8, 33L)+"/"+shiftByLongConst(8)); }
}
