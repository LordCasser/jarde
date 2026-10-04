import java.util.function.*;
public class L1 {
    private int field = 5;
    Runnable noCap(){ return () -> System.out.println("nc"); }              // 无捕获
    IntUnaryOperator capFinal(int k){ return x -> x + k; }                  // 捕获 final 局部(值)
    Supplier<Integer> capThis(){ return () -> field; }                      // 捕获 this
    Function<String,Integer> capTwo(String s, int n){ return t -> t.length() + s.length() + n + field; }
    public static void main(String[] a){ L1 o=new L1(); o.noCap().run(); System.out.println(o.capFinal(3).applyAsInt(4)+o.capThis().get()+o.capTwo("ab",1).apply("c")); }
}
