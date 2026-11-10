public class R1CatchLoopHoldout {
    static String trace = "";
    static int visit(int n, int i) { trace += "v"; if (i == 2 && n != 0) throw new ArithmeticException(); return n + i; }
    public static String run(int n) {
        trace = ""; int total = 0;
        try {
            for (int i = 0; i < Math.abs(n) % 4 + 1; i++) {
                try { total += visit(n, i); }
                catch (ArithmeticException e) { trace += "i"; total += 7; }
            }
            if (n < 0) throw new IllegalStateException();
        } catch (IllegalStateException e) { trace += "o"; total = -total; }
        return total + ":" + trace;
    }
}
