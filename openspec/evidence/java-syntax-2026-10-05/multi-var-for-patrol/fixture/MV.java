public class MV {
    static int twoVarFor(){ int c = 0; for(int i = 0, j = 10; i < j; i++, j--){ c++; } return c; }   // 双变量头（声明+更新双表达式）
    static int twoVarExisting(int n){ int c = 0, s = 0; for(c = 1, s = n; s > 0; c *= 2, s /= 2){ } return c; }  // 双既有变量（无声明）
    static int threeVar(){ int m = 0; for(int i = 0, j = 0, k = 5; i < k && j < k; i++, j += 2){ m = i + j; } return m; }  // 三变量+复合条件
    static int noInit(int[] xs){ int s = 0; int i = 0; for(; i < xs.length; i++){ s += xs[i]; } return s + i; }  // 空初始化段（i 循环后使用）
    static int emptyUpdate(int n){ int s = 0; for(int i = 0; i < n;){ s += i; i += 2; } return s; }  // 空更新段（更新在体内）
    public static void main(String[] a){ System.out.println(""+twoVarFor()+"/"+twoVarExisting(10)+"/"+threeVar()+"/"+noInit(new int[]{1,2,3})+"/"+emptyUpdate(6)); }
}
