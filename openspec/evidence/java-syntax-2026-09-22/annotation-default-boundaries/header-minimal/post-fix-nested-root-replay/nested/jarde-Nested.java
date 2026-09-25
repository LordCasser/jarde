// jarde: presentation of `Nested` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
@interface Nested {
    // jarde: no body: the member `child()LInner;` is declared abstract and its declaration carries no Code attribute
    public abstract Inner child() default @Inner(value = 6);

    // jarde: no body: the member `children()[LInner;` is declared abstract and its declaration carries no Code attribute
    public abstract Inner[] children() default {@Inner(value = 2), @Inner(value = 3)};
}
