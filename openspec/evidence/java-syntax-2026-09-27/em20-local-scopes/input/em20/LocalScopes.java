package em20;

public class LocalScopes {
    public static int joined(boolean positive, int value) {
        int result;
        if (positive) {
            result = value + 1;
        } else {
            result = value - 1;
        }
        return result;
    }

    public static int loop(int limit) {
        int sum = 0;
        for (int i = 0; i < limit; i++) {
            int current = i + 1;
            sum += current;
        }
        return sum;
    }

    public static int synchronizedLoop(int seed) {
        int limit;
        synchronized (LocalScopes.class) {
            limit = seed + 1;
        }
        int sum = 0;
        for (int i = 0; i < limit; i++) {
            sum += i;
        }
        return sum;
    }

    public static int caught(boolean fail) {
        int result = 1;
        try {
            if (fail) {
                throw new IllegalArgumentException("requested");
            }
            result = 2;
        } catch (IllegalArgumentException ex) {
            result = 3;
        }
        return result;
    }
}
