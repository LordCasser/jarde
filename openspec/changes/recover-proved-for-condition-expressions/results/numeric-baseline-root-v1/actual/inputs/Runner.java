package cf11;

public final class Runner {
    private static void runLong(String label, long[] values, long bound) {
        try {
            System.out.println(label + "=return:" + ForConditionNumeric.countLong(values, bound));
        } catch (RuntimeException error) {
            System.out.println(label + "=throw:" + error.getClass().getName());
        }
    }

    private static void runFloat(String label, float[] values, float bound) {
        try {
            System.out.println(label + "=return:" + ForConditionNumeric.countFloat(values, bound));
        } catch (RuntimeException error) {
            System.out.println(label + "=throw:" + error.getClass().getName());
        }
    }

    private static void runDouble(String label, double[] values, double bound) {
        try {
            System.out.println(label + "=return:" + ForConditionNumeric.countDouble(values, bound));
        } catch (RuntimeException error) {
            System.out.println(label + "=throw:" + error.getClass().getName());
        }
    }

    public static void main(String[] args) {
        runLong("long-normal", new long[] {1L, 4L}, 3L);
        runFloat("float-normal", new float[] {1.0f, 4.0f}, 3.0f);
        runDouble("double-normal", new double[] {1.0, 4.0}, 3.0);

        runFloat("float-nan-first", new float[] {Float.NaN, 4.0f}, 3.0f);
        runFloat("float-nan-later", new float[] {1.0f, Float.NaN}, 3.0f);
        runDouble("double-nan-first", new double[] {Double.NaN, 4.0}, 3.0);
        runDouble("double-nan-later", new double[] {1.0, Double.NaN}, 3.0);

        runLong("long-null", null, 3L);
        runFloat("float-null", null, 3.0f);
        runDouble("double-null", null, 3.0);
        runLong("long-bounds", new long[] {1L}, 3L);
        runFloat("float-bounds", new float[] {1.0f}, 3.0f);
        runDouble("double-bounds", new double[] {1.0}, 3.0);

        runLong("long-min-bound", new long[] {Long.MIN_VALUE, Long.MAX_VALUE}, Long.MIN_VALUE);
        runLong("long-max-bound", new long[] {Long.MIN_VALUE, Long.MAX_VALUE}, Long.MAX_VALUE);
    }
}
