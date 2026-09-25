package defpackage;

/* JADX INFO: loaded from: Defaults.class */
@interface Defaults {
    Inner nested() default @Inner(count = 9);

    Inner[] nestedArray() default {@Inner(count = 2), @Inner(count = 4)};

    Class<?> klass() default String[].class;

    Tone tone() default Tone.LOUD;

    int[] empty() default {};

    int[] numbers() default {3, 5};

    float negativeZeroFloat() default -0.0f;

    double negativeZeroDouble() default -0.0d;

    float positiveInfinity() default Float.POSITIVE_INFINITY;

    double notANumber() default Double.NaN;
}
