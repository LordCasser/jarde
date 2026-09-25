package defpackage;

/* JADX INFO: loaded from: FloatDefaults.class */
@interface FloatDefaults {
    float negativeZero() default -0.0f;

    double subnormal() default Double.MIN_VALUE;

    float positiveInfinity() default Float.NaN;

    double canonicalNaN() default Double.NaN;
}
