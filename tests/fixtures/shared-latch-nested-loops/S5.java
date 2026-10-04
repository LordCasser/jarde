public class S5 {
    // outer continue + inner loop, plain ints, no boxing/cast/concat
    public static int outerContinueInner(int n) {
        int s = 0;
        for (int i = 0; i < n; i++) {
            if (i % 2 == 0) continue;
            for (int j = 0; j < i; j++) { s += j; }
        }
        return s;
    }
    // outer continue, NO inner loop
    public static int outerContinueOnly(int n) {
        int s = 0;
        for (int i = 0; i < n; i++) { if (i % 2 == 0) continue; s += i; }
        return s;
    }
    // inner loop, NO outer continue
    public static int innerOnly(int n) {
        int s = 0;
        for (int i = 0; i < n; i++) { for (int j = 0; j < i; j++) { s += j; } }
        return s;
    }
    // outer continue + inner loop, labeled continue
    public static int labeledOuter(int n) {
        int s = 0;
        outer:
        for (int i = 0; i < n; i++) {
            if (i % 2 == 0) continue outer;
            for (int j = 0; j < i; j++) { s += j; }
        }
        return s;
    }
    public static void main(String[] a) {
        System.out.println(outerContinueInner(6));
        System.out.println(outerContinueOnly(6));
        System.out.println(innerOnly(6));
        System.out.println(labeledOuter(6));
    }
}
