enum Tone { SOFT, LOUD }
@interface Inner { int count() default 7; }
@interface Defaults {
  Inner nested() default @Inner(count = 9);
  Inner[] nestedArray() default {@Inner(count = 2), @Inner(count = 4)};
  Class<?> klass() default String[].class;
  Tone tone() default Tone.LOUD;
  int[] empty() default {};
  int[] numbers() default {3, 5};
  float negativeZeroFloat() default -0.0f;
  double negativeZeroDouble() default -0.0d;
  float positiveInfinity() default 1.0f / 0.0f;
  double notANumber() default 0.0d / 0.0d;
}
class DefaultsRunner {
  static void check(boolean ok, String what) { if (!ok) throw new AssertionError(what); }
  public static void main(String[] args) throws Exception {
    java.lang.reflect.Method m;
    m = Defaults.class.getMethod("nested");
    check(((Inner)m.getDefaultValue()).count() == 9, "nested annotation");
    m = Defaults.class.getMethod("nestedArray");
    Inner[] nested = (Inner[])m.getDefaultValue();
    check(nested.length == 2 && nested[0].count() == 2 && nested[1].count() == 4, "nested array");
    check(Defaults.class.getMethod("klass").getDefaultValue().equals(String[].class), "class literal");
    check(Defaults.class.getMethod("tone").getDefaultValue() == Tone.LOUD, "enum");
    check(((int[])Defaults.class.getMethod("empty").getDefaultValue()).length == 0, "empty array");
    check(java.util.Arrays.equals((int[])Defaults.class.getMethod("numbers").getDefaultValue(), new int[]{3,5}), "numbers");
    check(Float.floatToRawIntBits((Float)Defaults.class.getMethod("negativeZeroFloat").getDefaultValue()) == 0x80000000, "float negative zero");
    check(Double.doubleToRawLongBits((Double)Defaults.class.getMethod("negativeZeroDouble").getDefaultValue()) == 0x8000000000000000L, "double negative zero");
    check(((Float)Defaults.class.getMethod("positiveInfinity").getDefaultValue()).isInfinite(), "float infinity");
    check(((Double)Defaults.class.getMethod("notANumber").getDefaultValue()).isNaN(), "double NaN");
    System.out.println("reflection defaults: PASS");
  }
}
