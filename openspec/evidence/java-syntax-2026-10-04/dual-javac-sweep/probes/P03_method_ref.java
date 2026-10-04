import java.util.*;
import java.util.function.*;
public class P03_method_ref {
    static void run(List<String> l){ l.forEach(System.out::println); }
    static Supplier<String> sup(){ return P03_method_ref::make; }
    static String make(){ return "made"; }
    public static void main(String[] a){ run(Arrays.asList("x","y")); System.out.println(sup().get()); }
}
