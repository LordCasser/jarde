public final class ExceptionRegionsAudit {
    private static int effects;

    private static int work(int value) {
        effects++;
        if (value == 0) {
            throw new NumberFormatException("zero");
        }
        if (value == 2) {
            throw new IllegalStateException("two");
        }
        return value * 3;
    }

    private static int run() {
        effects = 0;
        int total = 0;
        for (int value = -1; value < 4; value++) {
            try {
                if (value < 0) {
                    effects++;
                    continue;
                }
                try {
                    total += work(value);
                } catch (NumberFormatException ex) {
                    effects += 10;
                    if (value == 0) {
                        continue;
                    }
                    total--;
                }
                if (value == 1) {
                    total += 100;
                }
                total += 5;
            } catch (IllegalStateException ex) {
                effects += 100;
                total += 2;
            }
        }
        return total;
    }

    public static void main(String[] args) {
        System.out.println(run() + ":" + effects);
    }
}
