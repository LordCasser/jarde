// jarde: presentation of `StringVarargs` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public enum StringVarargs {
    PAIR(".dex", ".class"),
    SINGLE(".xml"),
    EMPTY();

    private final java.lang.String[] exts;

    private StringVarargs(java.lang.String... arg0) {
        this.exts = arg0;
    }

    public java.lang.String[] valuesCopy() {
        // @method valuesCopy()[Ljava/lang/String;
        // @declaration an instance method of `StringVarargs`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.exts;
    }
}
