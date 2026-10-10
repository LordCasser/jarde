// jarde: presentation of `PrimitiveArrayBranches` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class PrimitiveArrayBranches extends java.lang.Object {
    public PrimitiveArrayBranches() {
        // @method <init>()V
        // @declaration a constructor of `PrimitiveArrayBranches`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static java.lang.Object test4(int arg0) {
        // @method test4(I)Ljava/lang/Object;
        // @declaration a static method of `PrimitiveArrayBranches`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 == 1) {
            return new int[]{1, 2};
        } else if (arg0 == 2) {
            return new float[]{0x1.000000p0f, 0x1.000000p1f};
    } else if (arg0 == 3) {
            return new short[]{1, 2};
    } else if (arg0 == 4) {
            return new byte[]{1, 2};
    } else {
            return null;
    }
    }

    public static java.lang.Object choose(int arg0) {
        // @method choose(I)Ljava/lang/Object;
        // @declaration a static method of `PrimitiveArrayBranches`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return test4(arg0);
    }
}
