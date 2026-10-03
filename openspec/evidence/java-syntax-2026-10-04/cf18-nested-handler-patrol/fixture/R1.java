public class R1 {
    // CF-18 core shape: nested handler crossing if + continue + loop back-edge
    public static int run(int n) {
        int total = 0;
        for (int i = 0; i < n; i++) {
            if (i < 0) {
                try {
                    total += risky(i);
                } catch (IllegalStateException e) {
                    continue;
                }
            } else {
                try {
                    total += risky(i);
                } catch (IllegalStateException e) {
                    total -= 1;
                }
            }
            if (total > 100) break;
        }
        return total;
    }
    static int risky(int v) {
        if (v % 2 == 0) throw new IllegalStateException("even");
        return v;
    }
    public static int nestedTry(int n) {
        int r = 0;
        try {
            for (int i = 0; i < n; i++) {
                try {
                    if (i == 2) continue;
                    r += risky(i);
                } catch (IllegalStateException e) {
                    r += 1;
                }
            }
        } catch (RuntimeException e) {
            r = -1;
        }
        return r;
    }
    public static void main(String[] a) {
        System.out.println(run(6));
        System.out.println(nestedTry(6));
    }
}
