import java.util.*;
import java.util.function.*;
public class P02_lambda {
    static int sum(List<Integer> l){ int[] t={0}; l.forEach(i -> t[0]+=i); return t[0]; }
    static String map(List<String> l){ StringBuilder s=new StringBuilder(); l.forEach(x -> s.append(x)); return s.toString(); }
    public static void main(String[] a){ System.out.println(sum(Arrays.asList(1,2,3))); System.out.println(map(Arrays.asList("a","b"))); }
}
