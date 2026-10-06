public class ICN {
    static int calls = 0;
    static String f(){ calls = calls + 1; return "f"; }
    static String g(){ calls = calls + 1; return "g"; }
    static int pick(int x){ return x + 1; }
    static void armCall(boolean flag){
        String s = "" + pick(1) + (flag ? f() : g()) + "/" + pick(2);
        System.out.println(s + calls);
    }
    static void armStore(boolean flag){
        int y = 0;
        String s = "" + pick(1) + (flag ? (y = 1) : (y = 2)) + "/" + pick(2);
        System.out.println(s + y);
    }
    static void nested(Object p, Object q, Object r, Object s){
        System.out.println("" + (p == q) + (r == s) + "/" + p.hashCode());
    }
    static void guarded(Object p, Object q){
        String s;
        try {
            s = "" + p.hashCode() + (p == q) + "/" + q.hashCode();
        } catch (RuntimeException e) {
            s = "caught";
        }
        System.out.println(s);
    }
}
