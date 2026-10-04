public class P01_string_concat {
    static String a(String x, int y){ return "v=" + x + ":" + y + "!"; }
    static String b(String x){ return x + x + x; }
    public static void main(String[] q){ System.out.println(a("s",3)); System.out.println(b("z")); }
}
