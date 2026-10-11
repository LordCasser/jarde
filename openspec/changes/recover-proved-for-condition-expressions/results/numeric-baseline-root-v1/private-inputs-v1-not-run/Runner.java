package cf11;

public final class Runner {
    private interface IntCall {
        int run();
    }

    private static void report(String name, IntCall call) {
        try {
            System.out.println(name + "=return:" + call.run());
        } catch (RuntimeException error) {
            System.out.println(name + "=throw:" + error.getClass().getName());
        }
    }

    public static void main(String[] args) {
        report("long-normal", () -> ForConditionNumeric.countLong(new long[] {1L, 4L}, 3L));
        report("float-normal", () -> ForConditionNumeric.countFloat(new float[] {1.0f, 4.0f}, 3.0f));
        report("double-normal", () -> ForConditionNumeric.countDouble(new double[] {1.0, 4.0}, 3.0));

        report("float-nan-first", () -> ForConditionNumeric.countFloat(new float[] {Float.NaN, 4.0f}, 3.0f));
        report("float-nan-later", () -> ForConditionNumeric.countFloat(new float[] {1.0f, Float.NaN}, 3.0f));
        report("double-nan-first", () -> ForConditionNumeric.countDouble(new double[] {Double.NaN, 4.0}, 3.0));
        report("double-nan-later", () -> ForConditionNumeric.countDouble(new double[] {1.0, Double.NaN}, 3.0));

        report("long-null", () -> ForConditionNumeric.countLong(null, 3L));
        report("float-null", () -> ForConditionNumeric.countFloat(null, 3.0f));
        report("double-null", () -> ForConditionNumeric.countDouble(null, 3.0));
        report("long-bounds", () -> ForConditionNumeric.countLong(new long[] {1L}, 3L));
        report("float-bounds", () -> ForConditionNumeric.countFloat(new float[] {1.0f}, 3.0f));
        report("double-bounds", () -> ForConditionNumeric.countDouble(new double[] {1.0}, 3.0));

        report("long-min-bound", () -> ForConditionNumeric.countLong(
                new long[] {Long.MIN_VALUE, Long.MAX_VALUE}, Long.MIN_VALUE));
        report("long-max-bound", () -> ForConditionNumeric.countLong(
                new long[] {Long.MIN_VALUE, Long.MAX_VALUE}, Long.MAX_VALUE));
    }
}
