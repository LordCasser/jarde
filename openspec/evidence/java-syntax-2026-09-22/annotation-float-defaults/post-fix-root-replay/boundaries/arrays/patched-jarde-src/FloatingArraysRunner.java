public final class FloatingArraysRunner {
    public static void main(String[] args) throws Exception {
        for (float value : (float[]) FloatingArrays.class.getDeclaredMethod("floats").getDefaultValue()) {
            System.out.printf("F=%08x%n", Float.floatToRawIntBits(value));
        }
        for (double value : (double[]) FloatingArrays.class.getDeclaredMethod("doubles").getDefaultValue()) {
            System.out.printf("D=%016x%n", Double.doubleToRawLongBits(value));
        }
    }
}
