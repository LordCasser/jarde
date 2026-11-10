public class C2AnonymousFactoryHoldout {
    static String trace = "";
    interface Part { String value(); }
    static Part first(int x) { return new Part() { { trace += "A"; } public String value() { trace += "a"; return "p" + x; } }; }
    static Part second(int x) { return new Part() { { trace += "B"; } public String value() { trace += "b"; return "q" + x; } }; }
    public static String run(int n) {
        trace = ""; Part[] parts = new Part[2];
        for (int i = 0; i < parts.length; i++) {
            final int item = n + i;
            parts[i] = i == 0 ? first(item) : second(item);
        }
        return parts[0].value() + ":" + parts[1].value() + ":" + trace;
    }
}
