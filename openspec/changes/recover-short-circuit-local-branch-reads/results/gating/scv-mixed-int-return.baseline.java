// jarde: presentation of `MixedIntReturn` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class MixedIntReturn extends java.lang.Object {
    static boolean bValue;

    static boolean cValue;

    public MixedIntReturn() {
        // @method <init>()V
        // @declaration a constructor of `MixedIntReturn`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean rhsB() {
        // @method rhsB()Z
        // @declaration a static method of `MixedIntReturn`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return MixedIntReturn.bValue;
    }

    static boolean rhsC() {
        // @method rhsC()Z
        // @declaration a static method of `MixedIntReturn`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return MixedIntReturn.cValue;
    }

    public static int value(boolean arg0) {
        // @method value(Z)I
        // @declaration a static method of `MixedIntReturn`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 ? !rhsB() ? rhsC() ? 1 : 0 : 1 : rhsC() ? 1 : 0;
    }
}
