public class R2NestedExitHoldout {
    static String trace = "";
    public static String run(int n) {
        trace = ""; int sum = 0;
        try {
            for (int x : new int[]{1, 2, 3}) {
                if (n == 0 && x == 2) { trace += "C"; continue; }
                for (int y = 0; y < 3; y++) {
                    trace += x + "" + y;
                    if (n == 1 && y == 1) break;
                    if (n == 4 && x == 2 && y == 1) return "R:" + trace;
                    if (n == 9 && x == 2 && y == 1) throw new UnsupportedOperationException();
                    sum += x * y;
                }
                if (n == -2 && x == 2) break;
            }
        } catch (UnsupportedOperationException e) { trace += "X"; return "T:" + trace; }
        return sum + ":" + trace;
    }
}
