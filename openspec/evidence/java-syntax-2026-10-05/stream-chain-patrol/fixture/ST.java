import java.util.*; import java.util.stream.*;
public class ST {
    static List<String> names(){ return Arrays.asList("alice","Bob","carol","dave"); }
    static List<String> uppers(){ return names().stream().map(String::toUpperCase).collect(Collectors.toList()); }   // unbound 实例方法引用
    static List<String> filtered(){ return names().stream().filter(s -> s.length() > 3).sorted().collect(Collectors.toList()); }  // lambda 谓词
    static int total(){ return Arrays.stream(new int[]{1,2,3,4}).filter(x -> x % 2 == 0).sum(); }   // IntStream
    static Map<Integer,List<String>> grouped(){ return names().stream().collect(Collectors.groupingBy(String::length)); }  // groupingBy
    static String joined(){ return names().stream().collect(Collectors.joining(",")); }
    public static void main(String[] a){ System.out.println(""+uppers()+"/"+filtered()+"/"+total()+"/"+grouped()+"/"+joined()); }
}
