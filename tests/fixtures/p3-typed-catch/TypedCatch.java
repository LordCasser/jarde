public class TypedCatch {
    public static int namedCatch(int n) {
        try {
            if (n < 0) throw new IllegalArgumentException();
            return n;
        } catch (IllegalArgumentException e) {
            return -1;
        }
    }

    public static int initialisedBeforeTry(int n) {
        int y = 2;
        int x = 1;
        try {
            if (n < 0) throw new IllegalArgumentException();
            return n;
        } catch (IllegalArgumentException e) {
            return x;
        }
    }

    public static int twoCatches(int n) {
        try {
            if (n < 0) throw new IllegalArgumentException();
            if (n > 10) throw new IllegalStateException();
            return n;
        } catch (IllegalArgumentException e) {
            return -1;
        } catch (IllegalStateException e) {
            return -2;
        }
    }

    public static int multiCatch(int n) {
        try {
            if (n < 0) throw new IllegalArgumentException();
            if (n > 10) throw new IllegalStateException();
            return n;
        } catch (IllegalArgumentException | IllegalStateException e) {
            return -3;
        }
    }

    public static int finallyIncrements(int n) {
        int x = 0;
        try {
            x = n;
        } finally {
            x = x + 1;
        }
        return x;
    }
}
