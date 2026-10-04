public class ST {
    static final String CS = "const-str";                 // 编译期常量 String（内联）
    static final int CI = 42;                              // 编译期常量 int
    static String vs = "var-str";                          // 非常量
    static String concat(){ return CS + "/" + CI + "/" + vs; }   // 常量折叠混合
    static long bigConst(){ return 123456789012345L; }     // ldc2_w long 常量
    static double dConst(){ return 3.14159; }
    static String interned(String s){ return s == "lit" ? "yes" : "no"; }  // 常量比较
    static String switchLike(String k){ return (k.hashCode()==104) ? "h" : "o"; } // hashCode 内联值
    public static void main(String[] a){ System.out.println(concat()+"/"+bigConst()+"/"+dConst()+"/"+interned("lit")+"/"+switchLike("h")); }
}
