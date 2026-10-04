public class IF {
    static int dense(boolean a, boolean b, int x){ return a ? b ? x+1 : x-1 : x*2; }    // 嵌套三元右侧
    static int chain(int n){ return n<0 ? -1 : n==0 ? 0 : n>9 ? 9 : n; }                 // 三元链（右结合）
    static boolean sc(boolean a, boolean b){ return (a && b) || (!a && !b); }            // 短路组合
    static int guard(int n){ if(n>0 && n<10) return 1; else if(n>=10 && n<100) return 2; else return 0; } // else-if 链+短路
    static Object poly(boolean c){ return c ? Integer.valueOf(1) : "s"; }                // 三元分支异型（装箱+String）
    public static void main(String[] a){ System.out.println(""+dense(true,false,5)+"/"+chain(15)+"/"+sc(true,true)+"/"+guard(50)+"/"+poly(false)); }
}
