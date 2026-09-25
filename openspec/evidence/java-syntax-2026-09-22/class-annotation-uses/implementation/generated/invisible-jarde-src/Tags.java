// jarde: presentation of `Tags` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
@java.lang.annotation.Retention(value = java.lang.annotation.RetentionPolicy.CLASS)

@interface Tags {
    // jarde: no body: the member `value()[LTag;` is declared abstract and its declaration carries no Code attribute
    public abstract Tag[] value();
}
