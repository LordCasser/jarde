public final class LoopIfJoinExtraBreak {
    public static int run(boolean outer, boolean inner) {
        int value = 0;
        if (outer) {
            if (inner) {
                value = 1;
            } else {
                while (value < 3) {
                    if (value == 1) break;
                    value++;
                }
            }
            value += 10;
        } else {
            value = 4;
        }
        return value;
    }

    public static int runReturn(boolean outer, boolean inner) {
        int value = 0;
        if (outer) {
            if (inner) {
                value = 1;
            } else {
                while (value < 3) {
                    if (value == 1) return 7;
                    value++;
                }
            }
            value += 10;
        } else {
            value = 4;
        }
        return value;
    }

    public static int runException(boolean outer, boolean inner) {
        int value = 0;
        if (outer) {
            if (inner) {
                value = 1;
            } else {
                while (value < 3) {
                    try {
                        value++;
                    } catch (RuntimeException ignored) {
                        value++;
                    }
                }
            }
            value += 10;
        } else {
            value = 4;
        }
        return value;
    }
}
