@interface FloatingBoundary {
    float negativeZero() default -0.0f;
    float leastSubnormal() default 0x0.000002p-126f;
    float greatestFinite() default 0x1.fffffep127f;
    float negativeInfinity() default Float.NEGATIVE_INFINITY;
    double leastSubnormalDouble() default 0x0.0000000000001p-1022d;
    double greatestFiniteDouble() default 0x1.fffffffffffffp1023d;
    double positiveInfinity() default Double.POSITIVE_INFINITY;
    double canonicalNaN() default Double.NaN;
}
