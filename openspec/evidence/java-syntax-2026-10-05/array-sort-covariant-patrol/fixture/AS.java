import java.util.*;
public class AS {
    static int[] sorted(int[] xs){ int[] c = xs.clone(); Arrays.sort(c); return c; }   // 数组 clone + sort（去重排序惯用法）
    static int max(int[] xs){ int m = xs[0]; for(int i = 1; i < xs.length; i++){ if(xs[i] > m){ m = xs[i]; } } return m; }   // 手写 max
    static String[] sortedStr(String[] xs){ Arrays.sort(xs); return xs; }              // 原地 sort 引用型
    static Number covariant(){ Object o = new Integer[3]; Integer[] back = (Integer[]) o; back[0] = 7; return back[0]; }   // 数组协变 + cast 往返
    static List<Integer> boxed(int[] xs){ List<Integer> l = new ArrayList<>(); for(int x : xs){ l.add(x); } return l; }   // 手动装箱
    public static void main(String[] a){ System.out.println(""+Arrays.toString(sorted(new int[]{3,1,2}))+"/"+max(new int[]{5,9,2})+"/"+Arrays.toString(sortedStr(new String[]{"b","a"}))+"/"+covariant()+"/"+boxed(new int[]{1,2})); }
}
