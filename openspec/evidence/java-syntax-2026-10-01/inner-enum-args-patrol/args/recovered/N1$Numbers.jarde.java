// jarde: presentation of `N1$Numbers` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `Ljava/lang/Enum<LN1$Numbers;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum N1$Numbers {
    ONE((byte) 1, N1$Numbers$NumString.ONE),
    TWO((byte) 2, N1$Numbers$NumString.TWO);

    private final byte num;

    private final N1$Numbers$NumString str;

    private N1$Numbers(byte arg0, N1$Numbers$NumString arg1) {
        this.num = arg0;
        this.str = arg1;
    }

    public int getNum() {
        // @method getNum()I
        // @declaration an instance method of `N1$Numbers`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.num;
    }

    public N1$Numbers$NumString getNumStr() {
        // @method getNumStr()LN1$Numbers$NumString;
        // @declaration an instance method of `N1$Numbers`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.str;
    }

    public java.lang.String getName() {
        // @method getName()Ljava/lang/String;
        // @declaration an instance method of `N1$Numbers`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.str.getName();
    }
}
