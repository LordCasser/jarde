public class R1LoopCatchDevB {
    static String trace = "";
    static int step(int x) { trace += "s"; if (x % 3 == 0) throw new IllegalArgumentException(); return x * 2; }
    public static String run(int n) {
        trace = ""; int total = 0;
        trace += "B";
        for (int i = 1; i <= Math.abs(n) % 5 + 1; i++) {
            { try { int result = step(n + i); total += result; total += result; trace += "o"; }
              catch (IllegalArgumentException e) { trace += "c"; total--; } }
        }
        trace += "b";
        return total + ":" + trace;
    }
}
