import java.util.*;
public class P02_multianewarray {
    static int sum(List<Integer> l){ int[][] t={{0}}; l.forEach(i -> t[0][0]+=i); return t[0][0]; }
    public static void main(String[] a){ System.out.println(sum(Arrays.asList(1,2,3))); }
}
