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
        // jarde: not recovered: the recovery run for `bothMatch(LTernaryInIfProbe;)Z` produced no statement (explanation only); the artifact's own comment lines are below
        // @method bothMatch(LTernaryInIfProbe;)Z
        // @declaration an instance method of `TernaryInIfProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 8 11 14 17 18 21 22 25 28 31 32 35 38 39 42 45 48 49 52 53 56 59 62 63 64 65
        // canonical block at BCI 62 on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted
    }
}
