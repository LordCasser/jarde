// jarde: presentation of `FloatingBoundary` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
@interface FloatingBoundary {
    // jarde: no body: the member `negativeZero()F` is declared abstract and its declaration carries no Code attribute
    public abstract float negativeZero() default -0.0f;

    // jarde: no body: the member `leastSubnormal()F` is declared abstract and its declaration carries no Code attribute
    public abstract float leastSubnormal() default 0x0.000002p-126f;

    // jarde: no body: the member `greatestFinite()F` is declared abstract and its declaration carries no Code attribute
    public abstract float greatestFinite() default 0x1.fffffep127f;

    // jarde: no body: the member `negativeInfinity()F` is declared abstract and its declaration carries no Code attribute
    public abstract float negativeInfinity() default Float.NEGATIVE_INFINITY;

    // jarde: no body: the member `leastSubnormalDouble()D` is declared abstract and its declaration carries no Code attribute
    public abstract double leastSubnormalDouble() default 0x0.0000000000001p-1022d;

    // jarde: no body: the member `greatestFiniteDouble()D` is declared abstract and its declaration carries no Code attribute
    public abstract double greatestFiniteDouble() default 0x1.fffffffffffffp1023d;

    // jarde: no body: the member `positiveInfinity()D` is declared abstract and its declaration carries no Code attribute
    public abstract double positiveInfinity() default Double.POSITIVE_INFINITY;

    // jarde: no body: the member `canonicalNaN()D` is declared abstract and its declaration carries no Code attribute
    public abstract double canonicalNaN() default Double.NaN;
}
