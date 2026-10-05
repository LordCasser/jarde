public class TC {
    static int deep(int x){ return x < 0 ? -1 : x == 0 ? 0 : x > 9 ? 9 : x; }   // 右结合链
    static String mixed(boolean a, boolean b){ return a ? (b ? "ab" : "a") : (b ? "b" : "n"); }  // 嵌套括号（同型 String）
    static int argPos(int x){ foo(x > 0 ? 1 : 2); return x; }                    // 三元作实参（int）
    static void foo(int v){ System.out.println("foo:" + v); }
    static String condCond(boolean a, boolean b){ return (a && b) ? "both" : (a || b) ? "one" : "none"; } // 复合条件+右链
    public static void main(String[] a){ foo(0); System.out.println(""+deep(15)+"/"+deep(-3)+"/"+mixed(true,false)+"/"+condCond(true,true)+"/"+condCond(false,true)); }
}
