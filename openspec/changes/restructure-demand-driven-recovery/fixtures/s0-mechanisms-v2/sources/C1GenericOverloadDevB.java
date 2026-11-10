public class C1GenericOverloadDevB {
    static String trace = "";
    static String choose(String s) { trace += "S"; return s.toUpperCase(); }
    static String choose(Integer i) { trace += "I"; return "#" + i; }
    static <T extends CharSequence> T echo(T value) { trace += "E"; return value; }
    public static String run(int n) {
        trace = "";
        trace += "B";
        String left, repeated; { String inferred = echo("v" + n); left = choose(inferred); repeated = choose(inferred); }
        String right = choose(Integer.valueOf(n));
        trace += "b";
        return left + ":" + repeated + ":" + right + ":" + trace;
    }
}
