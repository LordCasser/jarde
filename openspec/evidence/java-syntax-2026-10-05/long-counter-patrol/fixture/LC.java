public class LC {
    static long longSum(long n){ long s = 0; for(long i = 1; i <= n; i++){ s += i; } return s; }   // long 循环计数器
    static int longCmp(long a, long b, int x){ return a < b ? x : (a > b ? -x : 0); }               // lcmp 三向
    static double floatStep(int n){ double s = 0; for(int i = 0; i < n; i++){ s += 1.0 / (i + 1); } return s; }   // 循环内 double 累积
    static int narrow(long v){ int i = 0; if(v > Integer.MAX_VALUE){ i = Integer.MAX_VALUE; } else if(v < Integer.MIN_VALUE){ i = Integer.MIN_VALUE; } else { i = (int) v; } return i; }   // 显式收窄守卫惯用法
    public static void main(String[] a){ System.out.println(""+longSum(100)+"/"+longCmp(5,3,2)+"/"+longCmp(3,5,2)+"/"+longCmp(4,4,2)+"/"+floatStep(4)+"/"+narrow(5000000000L)+"/"+narrow(42)); }
}
