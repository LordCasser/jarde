public class PM {
    static final int READ = 1 << 0, WRITE = 1 << 1, EXEC = 1 << 2, ADMIN = 1 << 3;   // 掩码常量族
    static int grant(int cur, int p){ return cur | p; }                                // 纯表达式位或
    static int revoke(int cur, int p){ return cur & ~p; }                              // 清位
    static boolean can(int cur, int p){ return (cur & p) != 0; }                       // 测位
    static boolean canAll(int cur, int ps){ return (cur & ps) == ps; }                 // 全测
    static int fromOrdinal(int ord){ return 1 << ord; }                                // 位移生成
    public static void main(String[] a){
        int u = 0;
        u = grant(u, READ | EXEC);
        u = grant(u, WRITE);
        u = revoke(u, EXEC);
        System.out.println(""+can(u, READ)+"/"+can(u, EXEC)+"/"+canAll(u, READ | WRITE)+"/"+fromOrdinal(3)+"/"+u);
    }
}
