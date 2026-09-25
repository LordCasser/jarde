// jarde: presentation of `FloatDefaults` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
@interface FloatDefaults {
    // jarde: no body: the member `negativeZero()F` is declared abstract and its declaration carries no Code attribute
    public abstract float negativeZero();

    // jarde: no body: the member `subnormal()D` is declared abstract and its declaration carries no Code attribute
    public abstract double subnormal();

    // jarde: no body: the member `positiveInfinity()F` is declared abstract and its declaration carries no Code attribute
    public abstract float positiveInfinity();

    // jarde: no body: the member `canonicalNaN()D` is declared abstract and its declaration carries no Code attribute
    public abstract double canonicalNaN();
}
