// jarde: presentation of `MixedArrayValue` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class MixedArrayValue extends java.lang.Object {
    static boolean[] values;

    static boolean bValue;

    static boolean cValue;

    static int arrayCalls;

    static int indexCalls;

    static int bCalls;

    static int cCalls;

    public MixedArrayValue() {
        // @method <init>()V
        // @declaration a constructor of `MixedArrayValue`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static boolean[] array(boolean arg0) {
        // @method array(Z)[Z
        // @declaration a static method of `MixedArrayValue`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedArrayValue.arrayCalls = MixedArrayValue.arrayCalls + 1;
        return arg0 ? null : MixedArrayValue.values;
    }

    static int index(int arg0) {
        // @method index(I)I
        // @declaration a static method of `MixedArrayValue`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedArrayValue.indexCalls = MixedArrayValue.indexCalls + 1;
        return arg0;
    }

    static boolean b() {
        // @method b()Z
        // @declaration a static method of `MixedArrayValue`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedArrayValue.bCalls = MixedArrayValue.bCalls + 1;
        return MixedArrayValue.bValue;
    }

    static boolean c() {
        // @method c()Z
        // @declaration a static method of `MixedArrayValue`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedArrayValue.cCalls = MixedArrayValue.cCalls + 1;
        return MixedArrayValue.cValue;
    }

    static void one(boolean arg0, boolean arg1, int arg2) {
        // @method one(ZZI)V
        // @declaration a static method of `MixedArrayValue`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        array(arg1)[index(arg2)] = (arg0 ? !b() ? c() ? 1 : 0 : 1 : c() ? 1 : 0) % 2 != 0;
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `MixedArrayValue`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MixedArrayValue.values = new boolean[1];
    }
}
