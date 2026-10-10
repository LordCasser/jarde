public class Meet {
    static int at(String s, int k) {
        return s.charAt(k);
    }

    static int viaStore(char c) {
        int i = c;
        return i;
    }

    static int pass(int x) {
        return x;
    }

    static int fieldArg(String s) {
        return pass(s.charAt(0));
    }

    static long viaStoreLong(int n) {
        long l = n;
        return l;
    }

    static byte trunc(int n) {
        byte b = (byte) n;
        return b;
    }

    static char grade(int score) {
        switch (score) {
            case 90:
            case 95:
                return 'A';
            case 80:
                return 'B';
            default:
                return 'C';
        }
    }

    static int stat() {
        return 7;
    }

    static int viaRef(Meet m) {
        return m.stat();
    }

    static void unchecked(java.util.List raw) {
        raw.add("x");
    }

    /**
     * The `pop2` control (P3 2c.31): a call whose two-slot result nobody reads. javac discards it
     * with a `pop2`, and no shape of that rule accounts for one — `pop2` is not the `pop` of an
     * evaluated static-call qualifier and not the `pop` of a discarded one-slot result — so the
     * instruction keeps the quote it always had.
     */
    static void pop2Control() {
        System.nanoTime();
    }
}
