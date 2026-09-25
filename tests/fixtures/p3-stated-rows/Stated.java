public class Stated {
    static int steps(int n) {
        try {
            n = n + 1;
        } catch (RuntimeException e) {
            n = -1;
        }
        try {
            n = n * 2;
        } catch (IllegalStateException e) {
            n = -2;
        }
        return n;
    }

    static int nested(int n) {
        try {
            n = n + 1;
        } catch (RuntimeException e) {
            try {
                n = -1;
            } catch (IllegalArgumentException e2) {
                n = -2;
            }
        }
        return n;
    }
}
