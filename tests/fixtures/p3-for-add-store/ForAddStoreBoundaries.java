final class ForAddStoreBoundaries {
    static int changedStep(int limit, int step) {
        int sum = 0;
        int i = 0;
        while (i < limit) {
            sum += i;
            step = step + 1;
            i = i + step;
        }
        return sum;
    }

    static int extraConsumer(int limit, int step) {
        int sum = 0;
        int i = 0;
        while (i < limit) {
            sum += (i = i + step);
        }
        return sum;
    }

    static int sharedTailEffect(int limit, int step, int skip) {
        int sum = 0;
        int i = 0;
        while (i < limit) {
            if (i != skip) {
                sum += i;
            }
            sum += 100;
            i = i + step;
        }
        return sum;
    }

    static int observedAfter(int limit, int step) {
        int i = 0;
        while (i < limit) {
            i = i + step;
        }
        return i;
    }
}
