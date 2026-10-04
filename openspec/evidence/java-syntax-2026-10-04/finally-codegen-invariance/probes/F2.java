public class F2 {
    static int loopFin(int n) {                  // finally 内含循环
        int t = 0;
        try { for (int i = 0; i < n; i++) t += i; } finally { System.out.println("loopfin"); }
        return t;
    }
    static int finLoop(int n) {                  // 循环内含 finally
        int t = 0;
        for (int i = 0; i < n; i++) { try { t += i; } finally { if (i == 1) System.out.println("mid"); } }
        return t;
    }
    static int multiRet(int n) {                 // 多返回点 + finally
        try { if (n > 0) return 1; return -1; } finally { System.out.println("mr"); }
    }
    public static void main(String[] a){ System.out.println(loopFin(3)+finLoop(3)+multiRet(5)+multiRet(-1)); }
}
