// jarde: presentation of `TypeMark` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
@java.lang.annotation.Retention(value = java.lang.annotation.RetentionPolicy.RUNTIME)
@java.lang.annotation.Target(value = {java.lang.annotation.ElementType.TYPE_USE})

@interface TypeMark {
    // jarde: no body: the member `value()Ljava/lang/String;` is declared abstract and its declaration carries no Code attribute
    public abstract java.lang.String value();
}
