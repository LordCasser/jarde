// jarde: presentation of `MixedInstanceControls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class MixedInstanceControls extends java.lang.Object {
    static boolean bValue;

    static boolean cValue;

    static MixedInstanceControls$Box saved;

    static MixedInstanceControls$DerivedBox lastDerived;

    public MixedInstanceControls() {
        // @method <init>()V
        // @declaration a constructor of `MixedInstanceControls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static MixedInstanceControls$Box target(boolean arg0) {
        // @method target(Z)LMixedInstanceControls$Box;
        // @declaration a static method of `MixedInstanceControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new MixedInstanceControls$Box();
    }

    static MixedInstanceControls$DerivedBox derivedTarget(boolean arg0) {
        // @method derivedTarget(Z)LMixedInstanceControls$DerivedBox;
        // @declaration a static method of `MixedInstanceControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedInstanceControls.lastDerived = new MixedInstanceControls$DerivedBox();
        return MixedInstanceControls.lastDerived;
    }

    static boolean b() {
        // @method b()Z
        // @declaration a static method of `MixedInstanceControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return MixedInstanceControls.bValue;
    }

    static boolean c() {
        // @method c()Z
        // @declaration a static method of `MixedInstanceControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return MixedInstanceControls.cValue;
    }

    static void numeric(boolean arg0, boolean arg1) {
        // jarde: not recovered: the recovery run for `numeric(ZZ)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method numeric(ZZ)V
        // @declaration a static method of `MixedInstanceControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 5 8 11 14 17 20 21 24 25 28
        // the short-circuit chain from BCI 5 through 17 reaches a shared value consumer at BCI 25, but this slice has no SSA proof for that value; the complete region is quoted
        jarde_refused_body();
    }

    static boolean duplicated(boolean arg0, boolean arg1) {
        // jarde: not recovered: the recovery run for `duplicated(ZZ)Z` produced no statement (explanation only); the artifact's own comment lines are below
        // @method duplicated(ZZ)Z
        // @declaration a static method of `MixedInstanceControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 5 8 11 14 17 20 21 24 25 26 29
        // the short-circuit chain from BCI 5 through 17 reaches a shared value consumer at BCI 26, but this slice has no SSA proof for that value; the complete region is quoted
    }

    static void compound(boolean arg0, boolean arg1) {
        // jarde: not recovered: the recovery run for `compound(ZZ)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method compound(ZZ)V
        // @declaration a static method of `MixedInstanceControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 5 8 9 12 15 18 21 24 25 28 29 30 33
        // the short-circuit chain from BCI 9 through 21 reaches a shared value consumer at BCI 30, but this slice has no SSA proof for that value; the complete region is quoted
        jarde_refused_body();
    }

    static void sharedReceiver(boolean arg0, boolean arg1) {
        // jarde: not recovered: the recovery run for `sharedReceiver(ZZ)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method sharedReceiver(ZZ)V
        // @declaration a static method of `MixedInstanceControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 5 8 9 12 15 18 21 24 25 28 29 32
        // the short-circuit chain from BCI 9 through 21 reaches a shared value consumer at BCI 29, but this slice has no SSA proof for that value; the complete region is quoted
        jarde_refused_body();
    }

    static void protectedWrite(boolean arg0, boolean arg1) {
        // @method protectedWrite(ZZ)V
        // @declaration a static method of `MixedInstanceControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try {
            // @bytecode 0 1 4 5 8 11 14 17 20 21 24 25 28
            // block at BCI 0 leaves through exception handler 0: a handler's shape is not part of the recoverable subset
        } catch (java.lang.RuntimeException local2) {
        }
        return;
    }

    static void inheritedOwner(boolean arg0, boolean arg1) {
        // jarde: not recovered: the recovery run for `inheritedOwner(ZZ)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method inheritedOwner(ZZ)V
        // @declaration a static method of `MixedInstanceControls`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 5 8 11 14 17 20 21 24 25 28
        // the short-circuit chain from BCI 5 through 17 reaches a shared value consumer at BCI 25, but this slice has no SSA proof for that value; the complete region is quoted
        jarde_refused_body();
    }
}
