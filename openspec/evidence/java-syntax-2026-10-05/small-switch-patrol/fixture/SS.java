public class SS {
    static String one(String k){ switch(k){ case "a": return "1"; default: return "?"; } }            // 1 case String
    static String two(String k){ switch(k){ case "a": return "1"; case "b": return "2"; default: return "?"; } }  // 2 case
    static int oneInt(int k){ switch(k){ case 1: return 10; default: return 0; } }                     // 1 case int（tableswitch 照发）
    static int sparse(int k){ switch(k){ case 1: return 10; case 1000: return 20; default: return 0; } }  // 稀疏（lookupswitch）
    public static void main(String[] a){ System.out.println(""+one("a")+"/"+one("x")+"/"+two("b")+"/"+oneInt(1)+"/"+sparse(1000)); }
}
