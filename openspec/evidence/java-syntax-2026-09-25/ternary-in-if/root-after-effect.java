// jarde: presentation of `EffectProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class EffectProbe extends java.lang.Object {
    private final java.lang.String a;

    private final java.lang.String b;

    public EffectProbe(java.lang.String arg1, java.lang.String arg2) {
        // @method <init>(Ljava/lang/String;Ljava/lang/String;)V
        // @declaration a constructor of `EffectProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.a = arg1;
        this.b = arg2;
        return;
    }

    public boolean bothMatch(EffectProbe arg1) {
        // jarde: not recovered: the recovery run for `bothMatch(LEffectProbe;)Z` produced no statement (explanation only); the artifact's own comment lines are below
        // @method bothMatch(LEffectProbe;)Z
        // @declaration an instance method of `EffectProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 8 11 14 17 18 21 22 25 28 31 34 35 36 39 42 43 46 49 52 53 56 57 60 63 66 67 68 69
        // the shared terminal boolean return is not completely proved
    }
}
