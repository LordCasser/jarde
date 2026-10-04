import java.util.*;
public class P09_generics_bridge<T extends Comparable<T>> {
    T best(List<T> l){ T b=l.get(0); for(T x: l) if(x.compareTo(b)>0) b=x; return b; }
    public static void main(String[] a){ System.out.println(new P09_generics_bridge<Integer>().best(Arrays.asList(3,9,2))); }
}
