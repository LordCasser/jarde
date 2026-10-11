final class ComputedForControls {
    static int end;
    static int trace;
    static int mode;
    static final RuntimeException FAILURE = new IllegalStateException("chosen");

    static int f() {
        trace = trace * 10 + 1;
        if (mode == 1) throw FAILURE;
        return end;
    }

    static void g() {
        trace = trace * 10 + 2;
        end = 2;
        if (mode == 2) throw FAILURE;
    }

    static int find(int[] values, int target, int lower) {
        try {
            int i = f() - 1;
            g();
            while (i >= lower) {
                if (values[i] == target) return i;
                i--;
            }
            return -1;
        } catch (RuntimeException failure) {
            return failure == FAILURE ? -2 : -3;
        }
    }

    static int shared(int lower) {
        int seed;
        int i = (seed = f()) - 1;
        g();
        while (i >= lower) i--;
        return seed * 100 + i;
    }

    static int observedAfter(int lower) {
        int i = f() - 1;
        while (i >= lower) i--;
        return i;
    }

    static int externalWrite(int lower) {
        int i = f() - 1;
        while (i >= lower) i--;
        i = lower;
        return i;
    }

    static int scoped(int lower, boolean enter) {
        if (enter) {
            int i = f() - 1;
            while (i >= lower) i--;
            return i;
        }
        return -7;
    }
}
