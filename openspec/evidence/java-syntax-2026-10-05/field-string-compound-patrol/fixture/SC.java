public class SC {
    static String localCat(String a, int n){ String s = a; for(int i = 0; i < n; i++){ s += "-" + i; } return s; }   // 局部 String 复合循环
    String field; SC(String f){ field = f; }
    SC add(String x){ field += "[" + x + "]"; return this; }             // 字段 String 复合（链式）
    static String preAssign(String a){ String s = "p"; s += a; return s; }  // 单次复合
    static java.util.function.Function<Integer, java.util.function.Function<Integer, Integer>> curried(){   // 嵌套 lambda（柯里）
        return x -> y -> x + y;
    }
    static java.util.function.Supplier<Runnable> nested(){                // lambda 返回 lambda
        return () -> () -> System.out.println("nested-run");
    }
    static int useCurried(){ return curried().apply(3).apply(4); }
    public static void main(String[] a){ System.out.println(""+localCat("a", 3)+"/"+new SC("f").add("x").add("y").field+"/"+preAssign("q")+"/"+useCurried()); nested().get().run(); }
}
