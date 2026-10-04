public class D_varargs {
    static int sum(int... xs){ int t=0; for(int x:xs) t+=x; return t; }
    static String fmt(String f, Object... a){ return String.format(f, a); }
    public static void main(String[] x){ System.out.println(sum(1,2,3)+fmt("%s%s","a","b")); }
}
