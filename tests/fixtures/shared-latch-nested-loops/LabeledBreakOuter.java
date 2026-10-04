public class LabeledBreakOuter {
    public static int run(int n) {
        int s = 0;
        outer:
        for (int i = 0; i < n; i++) {
            if (i % 2 == 0) continue;
            for (int j = 0; j < i; j++) {
                if (j == 2) break outer;
                s += j;
            }
        }
        return s;
    }
    public static void main(String[] a) { System.out.println(run(6)); }
}
