public class IC {
    static boolean inCache(){ Integer a = 127, b = 127; return a == b; }        // 缓存内（引用相等为真——但字节码是引用比较）
    static boolean outCache(){ Integer a = 200, b = 200; return a == b; }       // 缓存外（引用不等——假）
    static boolean vsPrim(){ Integer a = 200; return a == 200; }                // 装箱 vs 字面量（一边拆箱→值比较真）
    static boolean longCmp(){ Long a = 200L, b = 200L; return a == b; }         // Long 同形
    static boolean doubleCmp(){ Double a = 1.0, b = 1.0; return a == b; }       // Double（无缓存恒假）
    static boolean unboxEq(){ Integer a = 200, b = 200; return a.equals(b) || a.intValue() == b; }  // equals/intValue
    public static void main(String[] a){ System.out.println(""+inCache()+"/"+outCache()+"/"+vsPrim()+"/"+longCmp()+"/"+doubleCmp()+"/"+unboxEq()); }
}
