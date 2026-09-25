public class NumericFlowRunner {
  public static void main(String[] args) {
    float[] floats = {Float.NaN, Float.NEGATIVE_INFINITY, -0.0f, 0.0f, Float.MIN_VALUE, Float.POSITIVE_INFINITY};
    double[] doubles = {Double.NaN, Double.NEGATIVE_INFINITY, -0.0d, 0.0d, Double.MIN_VALUE, Double.POSITIVE_INFINITY};
    for (int i = 0; i < floats.length; i++) {
      for (int j = 0; j < floats.length; j++) {
        System.out.println("float:" + i + ":" + j + "=" + NumericFlowAudit.floatNegated(floats[i], floats[j]));
        System.out.println("double:" + i + ":" + j + "=" + NumericFlowAudit.doubleChoice(doubles[i], doubles[j]));
        NumericEffects.trace = 0;
        NumericEffects.throwing = 0;
        System.out.println("ordered:" + i + ":" + j + "=" + NumericFlowAudit.ordered(floats[i], floats[j]) + ":" + NumericEffects.trace);
      }
    }
    long[] longs = {Long.MIN_VALUE, -1L, 0L, 1L, Long.MAX_VALUE};
    for (int i = 0; i < longs.length; i++) {
      for (int j = 0; j < longs.length; j++) {
        for (int k = 0; k < longs.length; k++) {
          System.out.println("long:" + i + ":" + j + ":" + k + "=" + NumericFlowAudit.longChain(longs[i], longs[j], longs[k]));
        }
      }
    }
    double[][] loops = {{0, 3, 1}, {4, 3, 1}, {Double.NaN, 3, 1}, {0, Double.NaN, 1}, {-0.0d, 0.0d, 1}, {0, 3, Double.POSITIVE_INFINITY}, {Double.NEGATIVE_INFINITY, Double.NEGATIVE_INFINITY, 1}};
    for (int i = 0; i < loops.length; i++) {
      System.out.println("loop:" + i + "=" + NumericFlowAudit.doubleLoop(loops[i][0], loops[i][1], loops[i][2]));
    }
    for (int throwing = 1; throwing <= 2; throwing++) {
      NumericEffects.trace = 0;
      NumericEffects.throwing = throwing;
      try {
        NumericFlowAudit.ordered(Float.NaN, Float.NaN);
        System.out.println("throw:" + throwing + "=returned");
      } catch (RuntimeException error) {
        System.out.println("throw:" + throwing + "=" + error.getClass().getName() + ":" + error.getMessage() + ":" + NumericEffects.trace);
      }
    }
  }
}
