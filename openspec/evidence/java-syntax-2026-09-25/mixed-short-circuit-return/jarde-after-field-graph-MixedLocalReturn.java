// jarde: presentation of `MixedLocalReturn` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class MixedLocalReturn extends java.lang.Object {
    static boolean bValue;

    static boolean cValue;

    static int bCalls;

    static int cCalls;

    public MixedLocalReturn() {
        // @method <init>()V
        // @declaration a constructor of `MixedLocalReturn`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean rhsB() {
        // @method rhsB()Z
        // @declaration a static method of `MixedLocalReturn`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedLocalReturn.bCalls = MixedLocalReturn.bCalls + 1;
        return MixedLocalReturn.bValue;
    }

    static boolean rhsC() {
        // @method rhsC()Z
        // @declaration a static method of `MixedLocalReturn`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedLocalReturn.cCalls = MixedLocalReturn.cCalls + 1;
        return MixedLocalReturn.cValue;
    }

    public static boolean value(boolean arg0) {
        // jarde: not recovered: the recovery run for `value(Z)Z` produced no statement (explanation only); the artifact's own comment lines are below
        // @method value(Z)Z
        // @declaration a static method of `MixedLocalReturn`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 10 13 16 17 20 21
        // canonical block at BCI 16 on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted
    }
}
