@interface Inner {
  int value() default 1;
}
@interface Nested {
  Inner child() default @Inner(value = 6);
  Inner[] children() default {@Inner(value = 2), @Inner(value = 3)};
}
