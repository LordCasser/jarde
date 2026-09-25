// jarde: presentation of `Defaults` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
@interface Defaults extends java.lang.annotation.Annotation {
    // jarde: no body: the member `nested()LInner;` is declared abstract and its declaration carries no Code attribute
    public abstract Inner nested();

    // jarde: no body: the member `nestedArray()[LInner;` is declared abstract and its declaration carries no Code attribute
    public abstract Inner[] nestedArray();

    // jarde: no body: the member `klass()Ljava/lang/Class;` is declared abstract and its declaration carries no Code attribute
    public abstract java.lang.Class klass() default java.lang.String[].class;

    // jarde: no body: the member `tone()LTone;` is declared abstract and its declaration carries no Code attribute
    public abstract Tone tone() default Tone.LOUD;

    // jarde: no body: the member `empty()[I` is declared abstract and its declaration carries no Code attribute
    public abstract int[] empty() default {};

    // jarde: no body: the member `numbers()[I` is declared abstract and its declaration carries no Code attribute
    public abstract int[] numbers() default {3, 5};

    // jarde: no body: the member `negativeZeroFloat()F` is declared abstract and its declaration carries no Code attribute
    public abstract float negativeZeroFloat();

    // jarde: no body: the member `negativeZeroDouble()D` is declared abstract and its declaration carries no Code attribute
    public abstract double negativeZeroDouble();

    // jarde: no body: the member `positiveInfinity()F` is declared abstract and its declaration carries no Code attribute
    public abstract float positiveInfinity();

    // jarde: no body: the member `notANumber()D` is declared abstract and its declaration carries no Code attribute
    public abstract double notANumber();
}
