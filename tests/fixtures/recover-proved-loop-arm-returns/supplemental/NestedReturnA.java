public final class NestedReturnA {
    public static final class TestCls {
        private int innerVisits;
        private int tailVisits;
        private int tailSum;
        private int earlyReturns;
        private int rolling;

        public int run(int outerLimit, int gate, int innerLimit, int returnOuter, int returnInner) {
            int localTail = 0;
            for (int i = outerLimit; i > 0; i--) {
                if (i > gate) {
                    int childSeed = i + 1;
                    int firstJ = 1;
                    for (int j = firstJ; j <= innerLimit; j++) {
                        int value = childSeed + j;
                        innerVisits++;
                        rolling = rolling * 31 + value;
                        if (i == returnOuter && j == returnInner) {
                            earlyReturns++;
                            return 0;
                        }
                    }
                }
                tailVisits++;
                tailSum += i;
                localTail += i;
                rolling = rolling * 37 + i;
            }
            return localTail;
        }
    }
}
