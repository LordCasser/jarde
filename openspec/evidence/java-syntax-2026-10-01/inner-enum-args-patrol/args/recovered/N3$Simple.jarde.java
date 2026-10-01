// jarde: presentation of `N3$Simple` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `Ljava/lang/Enum<LN3$Simple;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum N3$Simple {
    A((byte) 1, "x"),
    B((byte) 2, "y");

    private final byte num;

    private final java.lang.String s;

    private N3$Simple(byte arg0, java.lang.String arg1) {
        this.num = arg0;
        this.s = arg1;
    }

    public int n() {
        // @method n()I
        // @declaration an instance method of `N3$Simple`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.num;
    }
}
