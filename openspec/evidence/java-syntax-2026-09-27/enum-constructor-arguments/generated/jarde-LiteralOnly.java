// jarde: presentation of `LiteralOnly` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public enum LiteralOnly {
    FIRST(7),
    NEXT(-2);

    final int n;

    private LiteralOnly(int arg0) {
        this.n = arg0;
    }

    int value() {
        // @method value()I
        // @declaration an instance method of `LiteralOnly`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.n;
    }
}
