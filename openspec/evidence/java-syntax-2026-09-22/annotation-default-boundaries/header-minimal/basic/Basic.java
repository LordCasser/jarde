@interface Basic {
  int count() default 5;
  String label() default "source-only";
  int[] codes() default {2, 4};
}
