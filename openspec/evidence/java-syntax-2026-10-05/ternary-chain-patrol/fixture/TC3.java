public class TC3 {
    static String ifAnd(boolean a, boolean b){ if(a && b){ return "both"; } return "one"; }   // 短路+if（无三元）
    static String ifOr(boolean a, boolean b){ if(a || b){ return "1"; } return "0"; }
    public static void main(String[] a){ System.out.println(""+ifAnd(true,true)+"/"+ifOr(false,true)); }
}
