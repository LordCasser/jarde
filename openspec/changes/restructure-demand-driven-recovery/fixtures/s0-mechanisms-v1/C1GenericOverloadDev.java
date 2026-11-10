public class C1GenericOverloadDev {
    static String trace = "";
    static String choose(String s) { trace += "S"; return s.toUpperCase(); }
    static String choose(Integer i) { trace += "I"; return "#" + i; }
    static <T extends CharSequence> T echo(T value) { trace += "E"; return value; }
    public static String run(int n) {
        trace = "";
        String left = choose(echo("v" + n));
        String right = choose(Integer.valueOf(n));
        return left + ":" + right + ":" + trace;
    }
}
