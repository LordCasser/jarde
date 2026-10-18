public final class StatementfulLoopBoundary {
    public volatile int observed;
    public int calls;

    private int tick(int value) {
        calls++;
        return value;
    }

    public int extraBodyExit(int limit) {
        int i = 0;
        while ((this.observed = tick(i)) < limit) {
            if (i == 2) {
                break;
            }
            i++;
        }
        return i;
    }

    public int multipleLatches(int limit) {
        int i = 0;
        while ((this.observed = tick(i)) < limit) {
            if ((i & 1) == 0) {
                i++;
                continue;
            }
            i++;
        }
        return i;
    }
}
