// jarde: presentation of `IntArgs` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public enum IntArgs {
    LITERAL(1),
    FIELD(Ints.THREE),
    EXPR(Ints.THREE + 1);

    private final int n;

    private IntArgs(int arg0) {
        this.n = arg0;
    }

    public int value() {
        // @method value()I
        // @declaration an instance method of `IntArgs`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.n;
    }
}
