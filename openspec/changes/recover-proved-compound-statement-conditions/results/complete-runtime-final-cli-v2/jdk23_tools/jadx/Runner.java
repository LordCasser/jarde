import defpackage.CompoundStatementConditions;
public final class Runner {
    private Runner() {
    }

    private static String bits(boolean a, boolean b, boolean c, boolean d) {
        return (a ? "1" : "0")
                + (b ? "1" : "0")
                + (c ? "1" : "0")
                + (d ? "1" : "0");
    }

    private static void runBoolean(
            String id,
            String mode,
            boolean a,
            boolean b,
            boolean c,
            boolean d,
            String throwAt) {
        StringBuilder trace = new StringBuilder();
        String result = "<none>";
        String exception = "-";
        try {
            if ("plain".equals(mode)) {
                result = CompoundStatementConditions.test(a, b, c, d, trace);
            } else if ("mixed".equals(mode)) {
                result = CompoundStatementConditions.testMixed(a, b, c, d, trace);
            } else {
                result = CompoundStatementConditions.testThrowing(a, b, c, d, throwAt, trace);
            }
        } catch (RuntimeException failure) {
            result = "<threw>";
            exception = failure.getClass().getSimpleName() + ":" + failure.getMessage();
        }
        System.out.println(
                id + "|bits=" + bits(a, b, c, d)
                        + "|result=" + result
                        + "|trace=" + trace
                        + "|exception=" + exception);
    }

    private static void runFloat(String id, float value) {
        StringBuilder trace = new StringBuilder();
        String result = CompoundStatementConditions.testFloat(value, trace);
        System.out.println(id + "|result=" + result + "|trace=" + trace + "|exception=-");
    }

    public static void main(String[] args) {
        for (int mask = 0; mask < 16; mask++) {
            boolean a = (mask & 8) != 0;
            boolean b = (mask & 4) != 0;
            boolean c = (mask & 2) != 0;
            boolean d = (mask & 1) != 0;
            runBoolean("BOOL-PLAIN", "plain", a, b, c, d, null);
            runBoolean("BOOL-MIXED", "mixed", a, b, c, d, null);
            runBoolean("BOOL-THROW-NORMAL", "throw", a, b, c, d, null);
        }

        runBoolean("THROW-a", "throw", true, true, true, true, "a");
        runBoolean("THROW-b", "throw", true, true, true, true, "b");
        runBoolean("THROW-c", "throw", false, false, true, true, "c");
        runBoolean("THROW-d", "throw", false, false, true, true, "d");

        runBoolean("SKIP-THROW-b", "throw", false, false, true, true, "b");
        runBoolean("SKIP-THROW-c", "throw", true, true, true, true, "c");
        runBoolean("SKIP-THROW-d-after-first-true", "throw", true, true, true, true, "d");
        runBoolean("SKIP-THROW-d-after-c-false", "throw", false, false, false, true, "d");

        runFloat("FLOAT-nan", Float.NaN);
        runFloat("FLOAT-negative-infinity", Float.NEGATIVE_INFINITY);
        runFloat("FLOAT-positive-infinity", Float.POSITIVE_INFINITY);
        runFloat("FLOAT-below-threshold", -10.000001f);
        runFloat("FLOAT-threshold", -10.0f);
        runFloat("FLOAT-above-threshold", -9.999999f);
        runFloat("FLOAT-negative-zero", -0.0f);
        runFloat("FLOAT-positive-zero", 0.0f);
        runFloat("FLOAT-half", 0.5f);
    }
}
