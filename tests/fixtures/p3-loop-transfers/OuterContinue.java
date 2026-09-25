final class OuterContinue {
    static int run(int n) {
        int total = 0;
        outer:
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                if (j == 2) {
                    continue outer;
                }
                total++;
            }
            total += 100;
        }
        return total;
    }
}
