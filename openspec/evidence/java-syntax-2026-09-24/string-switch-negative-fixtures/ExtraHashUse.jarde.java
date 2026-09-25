// jarde: presentation of `ExtraHashUse` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ExtraHashUse extends java.lang.Object {
    static int calls;

    public ExtraHashUse() {
        // @method <init>()V
        // @declaration a constructor of `ExtraHashUse`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String read(java.lang.String arg0) {
        // @method read(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `ExtraHashUse`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        ExtraHashUse.calls = ExtraHashUse.calls + 1;
        return arg0;
    }

    public static int choose(java.lang.String arg0) {
        // @method choose(Ljava/lang/String;)I
        // @declaration a static method of `ExtraHashUse`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local3;
        int local4;
        int local5;
        java.lang.String local1 = read(arg0);
        int local2 = local1.hashCode();
        local3 = local2 & 1;
        local4 = -1;
        switch (local2) {
            case 2112:
                if (local1.equals((java.lang.Object) "Aa")) {
                    local4 = 0;
                } else {
                    if (local1.equals((java.lang.Object) "BB")) {
                        local4 = 1;
                    }
                }
                break;
            case 96354:
                if (local1.equals((java.lang.Object) "abc")) {
                    local4 = 2;
                }
                break;
        }
        switch (local4) {
            case 0:
                local5 = 10;
                break;
            case 1:
                local5 = 20;
                break;
            case 2:
                local5 = 30;
                break;
            default:
                local5 = 40;
                break;
        }
        return local5 + local3;
    }
}
