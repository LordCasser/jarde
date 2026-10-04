public class ExitDiverges {
    public static int run(int n) {
        int s = 0;
        for (int i = 0; i < n; i++) {
            if (i % 2 == 0) continue;
            for (int j = 0; j < i; j++) { s += j; }
            if (s > 100) { s -= 1; }
        }
        return s;
    }
    public static void main(String[] a) { System.out.println(run(6)); }
}
