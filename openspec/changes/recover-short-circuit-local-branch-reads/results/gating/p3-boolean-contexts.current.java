// jarde: presentation of `BooleanContexts` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class BooleanContexts extends java.lang.Object {
    public static boolean staticFlag;

    public BooleanContexts() {
        // @method <init>()V
        // @declaration a constructor of `BooleanContexts`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static boolean isZero(int arg0) {
        // @method isZero(I)Z
        // @declaration a static method of `BooleanContexts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 == 0) {
            return true;
        } else {
            return false;
        }
    }

    public static boolean flag() {
        // @method flag()Z
        // @declaration a static method of `BooleanContexts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return true;
    }

    public static int parity(int arg0) {
        // @method parity(I)I
        // @declaration a static method of `BooleanContexts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (flag()) {
            return 1;
        } else {
            return 0;
        }
    }

    public static boolean passed(boolean arg0) {
        // @method passed(Z)Z
        // @declaration a static method of `BooleanContexts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0;
    }

    public static boolean callFlag() {
        // @method callFlag()Z
        // @declaration a static method of `BooleanContexts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return flag();
    }

    public static boolean fieldFlag() {
        // @method fieldFlag()Z
        // @declaration a static method of `BooleanContexts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return BooleanContexts.staticFlag;
    }

    public static boolean localFromCall() {
        // @method localFromCall()Z
        // @declaration a static method of `BooleanContexts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        boolean local0 = flag();
        return local0;
    }

    public static boolean pick(int arg0, boolean arg1, boolean arg2) {
        // @method pick(IZZ)Z
        // @declaration a static method of `BooleanContexts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        boolean local3;
        if (arg0 != 0) {
            local3 = arg1;
        } else {
            local3 = arg2;
        }
        return local3;
    }

    public static boolean fromLocal(boolean arg0) {
        // @method fromLocal(Z)Z
        // @declaration a static method of `BooleanContexts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        boolean local1 = arg0;
        boolean local2 = local1;
        return local2;
    }

    public static boolean assignFromCall(boolean arg0) {
        // @method assignFromCall(Z)Z
        // @declaration a static method of `BooleanContexts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0 = flag();
        return arg0;
    }

    public static boolean throughLocal(boolean arg0) {
        // @method throughLocal(Z)Z
        // @declaration a static method of `BooleanContexts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        boolean local1 = arg0;
        return local1;
    }

    public static boolean negated(boolean arg0) {
        // @method negated(Z)Z
        // @declaration a static method of `BooleanContexts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return !arg0;
    }

    public static int staticFlagCount() {
        // @method staticFlagCount()I
        // @declaration a static method of `BooleanContexts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (BooleanContexts.staticFlag) {
            return 1;
        } else {
            return 0;
        }
    }

    public static int intLocal(int arg0) {
        // @method intLocal(I)I
        // @declaration a static method of `BooleanContexts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        local1 = 0;
        if (arg0 != 0) {
            local1 = 1;
        } else {
            local1 = arg0;
        }
        return local1;
    }

    public static int count(boolean arg0) {
        // @method count(Z)I
        // @declaration a static method of `BooleanContexts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0) {
            return 1;
        } else {
            return 0;
        }
    }

    public static int nonzero(int arg0) {
        // @method nonzero(I)I
        // @declaration a static method of `BooleanContexts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 != 0) {
            return 1;
        } else {
            return 0;
        }
    }

    public static int answer() {
        // @method answer()I
        // @declaration a static method of `BooleanContexts`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 1;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `BooleanContexts`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        BooleanContexts.staticFlag = true;
    }
}
