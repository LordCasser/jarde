public class Grid {
    static int nestedBreak(int n) {
        int total = 0;
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                if (i + j > 5) {
                    break;
                }
                total = total + 1;
            }
        }
        return total;
    }

    static int labeledContinue(int n) {
        int total = 0;
        outer:
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                if (j == 2) {
                    continue outer;
                }
                total = total + 1;
            }
        }
        return total;
    }

    static int labeledBreak(int n) {
        int total = 0;
        outer:
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                if (j == 3) {
                    break outer;
                }
                total = total + 1;
            }
            total = total + 10;
        }
        return total;
    }
}
