class FloatRunner {
  public static void main(String[] args) throws Exception {
    System.out.println(Integer.toHexString(Float.floatToRawIntBits((Float) FloatDefaults.class.getDeclaredMethod("negativeZero").getDefaultValue())));
    System.out.println(Long.toHexString(Double.doubleToRawLongBits((Double) FloatDefaults.class.getDeclaredMethod("subnormal").getDefaultValue())));
    System.out.println(Integer.toHexString(Float.floatToRawIntBits((Float) FloatDefaults.class.getDeclaredMethod("positiveInfinity").getDefaultValue())));
    System.out.println(Long.toHexString(Double.doubleToRawLongBits((Double) FloatDefaults.class.getDeclaredMethod("canonicalNaN").getDefaultValue())));
  }
}
