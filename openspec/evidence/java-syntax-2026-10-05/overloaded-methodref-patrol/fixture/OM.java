import java.util.function.*;
public class OM {
    static String pickStr(Function<Integer,String> f){ return f.apply(7); }         // String::valueOf → valueOf(int)?
    static int pickInt(Function<String,Integer> g){ return g.apply("42"); }          // Integer::valueOf(String)
    static Integer pickIdent(Function<Integer,Integer> h){ return h.apply(9); }      // Integer::valueOf(int) 装箱链
    static String overloaded(Function<Integer,String> f){ return f.apply(3); }       // 自建重载族方法引用
    static String o(int v){ return "i"+v; }
    static String o(String v){ return "s"+v; }
    static String o(Object v){ return "o"+v; }
    public static void main(String[] a){
        System.out.println(pickStr(String::valueOf)+"/"+pickInt(Integer::valueOf)+"/"+pickIdent(Integer::valueOf)+"/"+overloaded(OM::o)+"/"+overloaded(x -> "L"+x));
    }
}
