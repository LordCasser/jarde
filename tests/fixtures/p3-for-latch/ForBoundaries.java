final class ForBoundaries {
    static int counted(int n) {
        int sum = 0;
        for (int i = 0; i < n; i++) {
            sum += i;
        }
        return sum;
    }

    static int observedAfter(int n) {
        int i = 0;
        while (i < n) {
            i++;
        }
        return i;
    }

    static int twoUpdates(int n) {
        int sum = 0;
        for (int i = 0; i < n; i++) {
            sum += i;
            i++;
        }
        return sum;
    }

    static int varyingBound(int n) {
        int sum = 0;
        int bound = n;
        for (int i = 0; i < bound; i++) {
            sum++;
            bound--;
        }
        return sum;
    }

    static int noUpdate(int n) {
        int i = 0;
        while (i < n) {
            n--;
        }
        return n;
    }
}
