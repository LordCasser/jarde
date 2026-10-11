public final class ComputedForEffectControl {
    static int end;
    static int mode;
    static int fCalls;
    static int gCalls;
    static int order;

    static void reset(int nextMode) {
        end = 3;
        mode = nextMode;
        fCalls = 0;
        gCalls = 0;
        order = 0;
    }

    static int f() {
        fCalls++;
        order = order * 10 + 1;
        if (mode == 1) {
            throw new RuntimeException("fail-f");
        }
        return end;
    }

    static void g() {
        gCalls++;
        order = order * 10 + 2;
        end = 0;
        if (mode == 2) {
            throw new RuntimeException("fail-g");
        }
    }

    static int find(int[] values, int target, int lower, int pad0, int pad1, int pad2) {
        int i = f() - 1;
        g();
        while (i >= lower) {
            if (values[i] == target) {
                return i;
            }
            i--;
        }
        return -1;
    }
}
