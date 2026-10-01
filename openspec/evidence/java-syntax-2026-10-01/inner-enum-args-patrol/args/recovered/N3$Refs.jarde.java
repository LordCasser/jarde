// jarde: presentation of `N3$Refs` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `Ljava/lang/Enum<LN3$Refs;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum N3$Refs {
    A(N3$Simple.A),
    B(N3$Simple.B);

    private final N3$Simple s;

    private N3$Refs(N3$Simple arg0) {
        this.s = arg0;
    }

    public N3$Simple g() {
        // @method g()LN3$Simple;
        // @declaration an instance method of `N3$Refs`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.s;
    }
}
