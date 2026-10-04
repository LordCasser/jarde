public class F1 {
    static int plain(String s) {                 // 基本 finally + return in try
        try { return s.length(); } finally { System.out.println("fin1"); }
    }
    static int noRet(String s) {                 // finally 无 return-in-try
        try { System.out.println(s); } finally { System.out.println("fin2"); }
        return 1;
    }
    static int catchFin(String s) {              // try/catch/finally 三层
        try { return s.length(); } catch (RuntimeException e) { return -1; } finally { System.out.println("fin3"); }
    }
    static int nested(String s) {                // 嵌套 try-finally
        try { try { return s.length(); } finally { System.out.println("inner"); } } finally { System.out.println("outer"); }
    }
    public static void main(String[] a){ System.out.println(plain("ab")+noRet("x")+catchFin("cd")+nested("ef")); }
}
