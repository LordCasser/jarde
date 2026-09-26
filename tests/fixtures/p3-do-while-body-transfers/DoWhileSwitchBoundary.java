public final class DoWhileSwitchBoundary {
    public static int switchBreak(int limit) {
        int i = 0;
        int trace = 0;
        do {
            i++;
            switch (i) {
                case 2:
                    break;
                default:
                    trace = trace * 10 + i;
            }
            trace = trace * 10 + 9;
        } while (i < limit);
        return trace;
    }
}
