// jarde: presentation of `IntegerSwitchAudit` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class IntegerSwitchAudit extends java.lang.Object {
    static final int LOW = 2748;

    static final int HIGH = 3294;

    public IntegerSwitchAudit() {
        // @method <init>()V
        // @declaration a constructor of `IntegerSwitchAudit`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int grouped(int arg0) {
        // @method grouped(I)I
        // @declaration a static method of `IntegerSwitchAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        switch (arg0) {
            case 1:
            case 7:
                return 11;
            case 2:
                return 20;
            default:
                return -1;
        }
    }

    public static int fallthrough(int arg0) {
        // @method fallthrough(I)I
        // @declaration a static method of `IntegerSwitchAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        local1 = 3;
        switch (arg0) {
            case 10:
                local1 = local1 + 4;
            case 20:
                local1 = local1 * 2;
                break;
            case 30:
                local1 = -5;
                break;
        }
        return local1;
    }

    public static int noDefault(int arg0) {
        // @method noDefault(I)I
        // @declaration a static method of `IntegerSwitchAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        local1 = 99;
        switch (arg0) {
            case 4:
                local1 = 2748;
                break;
            case 8:
                local1 = 3294;
                break;
        }
        return local1;
    }

    public static int labelConstant(int arg0) {
        // @method labelConstant(I)I
        // @declaration a static method of `IntegerSwitchAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        switch (arg0) {
            case 2748:
                return 3294;
            default:
                return 0;
        }
    }
}
