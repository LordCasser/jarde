// jarde: presentation of `SharedTrueShortCircuit` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class SharedTrueShortCircuit extends java.lang.Object {
    static boolean result;

    static int calls;

    public SharedTrueShortCircuit() {
        // @method <init>()V
        // @declaration a constructor of `SharedTrueShortCircuit`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean rhs() {
        // @method rhs()Z
        // @declaration a static method of `SharedTrueShortCircuit`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        SharedTrueShortCircuit.calls = SharedTrueShortCircuit.calls + 1;
        return true;
    }

    static void assign(boolean arg0) {
        // @method assign(Z)V
        // @declaration a static method of `SharedTrueShortCircuit`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        SharedTrueShortCircuit.result = (!arg0 ? rhs() ? 1 : 0 : 1) % 2 != 0;
        return;
    }
}
