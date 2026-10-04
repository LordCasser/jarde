public class Overlap {
    public static int run(int n) {
        int s = 0;
        for (int i = 0; i < n; i++) {
            if (i % 2 == 0) continue;
            for (int j = 0; j < i; j++) {
                if (j == 2) break;
                if (j == 1) continue;
                s += j * 10;
            }
            s += 100;
        }
        return s;
    }
    public static void main(String[] a) { System.out.println(run(6)); }
}
