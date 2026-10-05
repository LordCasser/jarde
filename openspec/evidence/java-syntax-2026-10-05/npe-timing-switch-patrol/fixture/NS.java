public class NS {
    static String nullStr(String k){ switch(k){ case "a": return "1"; default: return "d"; } }   // switch(null String) → NPE at hashCode
    enum E { A, B }
    static String nullEnum(E k){ switch(k){ case A: return "1"; default: return "d"; } }          // switch(null enum) → NPE at ordinal
    static String viaIf(String k){ if("a".equals(k)){ return "1"; } return "d"; }                 // 对照：if 形对 null 安全
    static int nullBoxed(Integer k){ switch(k){ case 1: return 1; default: return 0; } }          // switch(null Integer) → NPE at intValue
    public static void main(String[] a){
        System.out.println(viaIf(null));
        try { nullStr(null); } catch(NullPointerException e){ System.out.println("NPE-str"); }
        try { nullEnum(null); } catch(NullPointerException e){ System.out.println("NPE-enum"); }
        try { nullBoxed(null); } catch(NullPointerException e){ System.out.println("NPE-boxed"); }
    }
}
