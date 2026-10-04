import java.util.*;
public class D_diamond {
    static List<String> mk(){ return new ArrayList<>(); }              // 菱形
    static Map<String,List<Integer>> mk2(){ return new HashMap<>(); }   // 嵌套菱形
    public static void main(String[] a){ System.out.println(mk().size()+mk2().size()); }
}
