import java.util.*;
public class C_foreach {
    static int sum(int[] xs){ int t=0; for(int x: xs) t+=x; return t; }
    static int sumL(List<Integer> l){ int t=0; for(int x: l) t+=x; return t; }
    public static void main(String[] a){ System.out.println(sum(new int[]{1,2,3})+"/"+sumL(Arrays.asList(4,5))); }
}
