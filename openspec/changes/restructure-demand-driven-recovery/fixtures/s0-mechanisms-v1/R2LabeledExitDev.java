public class R2LabeledExitDev {
    static String trace = "";
    public static String run(int n) {
        trace = ""; int total = 0;
        try {
            outer: for (int i = 0; i < 3; i++) {
                for (int j = 0; j < 4; j++) {
                    trace += i + "" + j;
                    if (n == 0 && j == 1) continue outer;
                    if (n == 1 && i == 1) break outer;
                    if (n == 4 && i == 1 && j == 2) return "ret:" + trace;
                    if (n == 9 && i == 1 && j == 1) throw new IllegalArgumentException();
                    total += i + j;
                }
            }
        } catch (IllegalArgumentException e) { trace += "caught"; return "throw:" + trace; }
        return total + ":" + trace;
    }
}
