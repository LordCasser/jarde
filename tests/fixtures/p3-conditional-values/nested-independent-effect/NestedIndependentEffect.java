final class NestedIndependentEffect {
    static int calls;

    static void mark() {
        calls++;
    }

    static int choose(boolean outer, boolean inner) {
        int value;
        if (outer) {
            mark();
            if (inner) {
                value = 2;
            } else {
                value = 3;
            }
        } else {
            value = 1;
        }
        return value;
    }
}
