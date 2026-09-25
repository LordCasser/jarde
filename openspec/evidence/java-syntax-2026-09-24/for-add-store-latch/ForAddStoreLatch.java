public final class ForAddStoreLatch {
    private ForAddStoreLatch() {
    }

    public static int run(int limit, int step, int skipAt) {
        int index;
        int indexTotal = 0;
        int afterInnerCount = 0;

        outer:
        for (index = 0; index < limit; index = index + step) {
            indexTotal += index;
            for (int inner = 0; inner < 2; inner++) {
                if (index == skipAt) {
                    continue outer;
                }
            }
            afterInnerCount++;
        }

        return indexTotal + afterInnerCount * 10000;
    }
}
