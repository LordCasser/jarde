// jarde: presentation of `LocalShortCircuit` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class LocalShortCircuit extends java.lang.Object {
    static java.lang.String observed;

    static boolean visible;

    LocalShortCircuit() {
        // @method <init>()V
        // @declaration a constructor of `LocalShortCircuit`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static void check(boolean gate) {
        // @method check(Z)V
        // @declaration a static method of `LocalShortCircuit`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String expected = "captured-value";
        LocalShortCircuit.visible = gate && expected.equals((java.lang.Object) LocalShortCircuit.observed);
        return;
    }
}
