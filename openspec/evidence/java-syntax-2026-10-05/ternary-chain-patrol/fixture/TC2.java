public class TC2 {
    static String plainAnd(boolean a, boolean b){ return (a && b) ? "both" : "one"; }     // 短路+三元（无右链）
    static String chain(boolean a, boolean b){ return a ? "x" : b ? "y" : "n"; }          // 右链（无短路）
    static String shortOnly(boolean a, boolean b){ return (a && b) ? "1" : (a || b) ? "2" : "3"; } // 复现探针（=condCond）
    static String andOr(boolean a, boolean b, boolean c){ return ((a || b) && c) ? "y" : "n"; } // 混合括号短路
    public static void main(String[] a){ System.out.println(""+plainAnd(true,true)+"/"+chain(false,false)+"/"+andOr(false,true,true)); }
}
