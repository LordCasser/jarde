// jarde: presentation of `TernaryInIfProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class TernaryInIfProbe extends java.lang.Object {
    private final java.lang.String a;

    private final java.lang.String b;

    public TernaryInIfProbe(java.lang.String arg1, java.lang.String arg2) {
        // @method <init>(Ljava/lang/String;Ljava/lang/String;)V
        // @declaration a constructor of `TernaryInIfProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.a = arg1;
        this.b = arg2;
        return;
    }

    public boolean bothMatch(TernaryInIfProbe arg1) {
        // @method bothMatch(LTernaryInIfProbe;)Z
        // @declaration an instance method of `TernaryInIfProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.a == null ? arg1.a == null ? this.b == null ? arg1.b == null ? true : false : this.b.equals((java.lang.Object) arg1.b) ? true : false : false : this.a.equals((java.lang.Object) arg1.a) ? this.b == null ? arg1.b == null ? true : false : this.b.equals((java.lang.Object) arg1.b) ? true : false : false;
    }
}
