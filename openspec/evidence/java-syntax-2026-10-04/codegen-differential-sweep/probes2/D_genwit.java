import java.util.*;
public class D_genwit {
    static <T> List<T> id(List<T> l){ return l; }
    static <T> T first(List<T> l){ return l.get(0); }
    public static void main(String[] x){ List<String> l=Arrays.asList("a","b"); System.out.println(D_genwit.<String>id(l).size()+first(l)); }  // 显式类型见证
}
