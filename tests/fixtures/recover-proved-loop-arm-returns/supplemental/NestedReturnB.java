public final class NestedReturnB {
    public static final class TestCls {
        private int innerVisits;
        private int tailVisits;
        private int tailSum;
        private int earlyReturns;
        private int rolling;

        public int run(int returnOuter, int innerLimit, int gate, int returnInner, int outerLimit) {
            int localTail = 0;
            for (int i = outerLimit; i > 0; i--) {
                if (i > gate) {
                    int childSeed = i + 2;
                    int firstJ = 0;
                    for (int j = firstJ; j < innerLimit; j++) {
                        int value = childSeed + j;
                        innerVisits++;
                        rolling = rolling * 29 + value;
                        if (i == returnOuter && j == returnInner) {
                            earlyReturns++;
                            return 0;
                        }
                    }
                }
                tailVisits++;
                tailSum += i;
                localTail += i;
                rolling = rolling * 41 + i;
            }
            return localTail;
        }
    }
}
