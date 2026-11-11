// jarde: presentation of `N0` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `Ljava/lang/Enum<LN0;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum N0 {
    A("a"),
    B("b");

    private final java.lang.String s;

    private N0(java.lang.String arg0) {
        this.s = arg0;
    }

    public java.lang.String get() {
        // @method get()Ljava/lang/String;
        // @declaration an instance method of `N0`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.s;
    }
}
