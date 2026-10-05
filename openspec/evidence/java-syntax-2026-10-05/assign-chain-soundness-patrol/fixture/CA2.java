public class CA2 {
    static int a, b, c;
    static int chained(){ a = b = c = 5; return a + b + c; }             // 链式纯赋值 a=b=c
    static int[] arr = new int[4];
    static int chainedArray(){ arr[1] = arr[2] = 7; return arr[1] + arr[2]; }   // 链式数组存储
    static String nullSafe(String y){ return y == null ? "dflt" : y.trim(); }   // null 安全三元惯用法
    static int condArg(int x, int y){ return Math.max(x > 0 ? x : -x, y); }     // 三元作实参
    static int assignInBranch(int[] xs, boolean flag){ int r = 0; if(flag){ r = xs[0]; } else { r = xs[1]; } return r; }   // 双分支赋值
    public static void main(String[] a){ System.out.println(""+chained()+"/"+chainedArray()+"/"+nullSafe(null)+"/"+nullSafe(" q ")+"/"+condArg(-3, 2)+"/"+assignInBranch(new int[]{9, 4}, true)); }
}
