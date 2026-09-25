// jarde: presentation of `ActualStackJoin` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ActualStackJoin extends java.lang.Object {
    private ActualStackJoin() {
        // @method <init>()V
        // @declaration a constructor of `ActualStackJoin`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int runByte(int arg0) {
        // @method runByte(I)I
        // @declaration a static method of `ActualStackJoin`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        switch (arg0) {
            case 1:
                local1 = 130;
                break;
            default:
                local1 = -129;
                break;
        }
        return local1;
    }

    public static int runChar(int arg0) {
        // @method runChar(I)I
        // @declaration a static method of `ActualStackJoin`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        switch (arg0) {
            case 1:
                local1 = 65535;
                break;
            default:
                local1 = -2;
                break;
        }
        return local1;
    }

    public static int runShort(int arg0) {
        // @method runShort(I)I
        // @declaration a static method of `ActualStackJoin`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        switch (arg0) {
            case 1:
                local1 = 32768;
                break;
            default:
                local1 = -32769;
                break;
        }
        return local1;
    }
}
