final class LoopTestValues {
    int n;

    int fieldThenIncrementTest(int local) {
        while (this.n > 0) {
            local++;
        }
        while (local-- > 0) {
        }
        return local;
    }

    int storeTest(int local) {
        while ((local = -1) > 0) {
        }
        return local;
    }

    int incrementTest(int local) {
        while (local-- > 0) {
        }
        return local;
    }
}
