// jarde: presentation of `Basic` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
@interface Basic extends java.lang.annotation.Annotation {
    // jarde: no body: the member `count()I` is declared abstract and its declaration carries no Code attribute
    public abstract int count() default 5;

    // jarde: no body: the member `label()Ljava/lang/String;` is declared abstract and its declaration carries no Code attribute
    public abstract java.lang.String label() default "source-only";

    // jarde: no body: the member `codes()[I` is declared abstract and its declaration carries no Code attribute
    public abstract int[] codes() default {2, 4};
}
