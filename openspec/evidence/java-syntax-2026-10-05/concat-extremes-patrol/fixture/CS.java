public class CS {
    static String nullCat(){ return "pre" + null + "post"; }             // null 拼接（"null" 字面量化）
    static String charCat(){ return "" + 'x' + 'y'; }                     // char 序列拼接
    static String mixed(){ return "n=" + 1 + " c=" + 'c' + " b=" + true + " nul=" + null; }  // 全类型混合
    static String intInMiddle(){ return 1 + 2 + "=3" ; }                  // int 先算后接
    static String strFirst(){ return "1" + 2 + 3; }                       // String 先行（全接）
    static String selfRef(){ String s = "a"; s = s + "b" + s; return s; } // 自引用拼接
    public static void main(String[] a){ System.out.println(""+nullCat()+"/"+charCat()+"/"+mixed()+"/"+intInMiddle()+"/"+strFirst()+"/"+selfRef()); }
}
