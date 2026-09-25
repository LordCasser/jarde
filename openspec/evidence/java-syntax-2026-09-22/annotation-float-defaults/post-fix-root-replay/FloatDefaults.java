@interface FloatDefaults {
  float negativeZero() default -0.0f;
  double subnormal() default 0x0.0000000000001p-1022;
  float positiveInfinity() default 1.0f / 0.0f;
  double canonicalNaN() default 0.0d / 0.0d;
}
