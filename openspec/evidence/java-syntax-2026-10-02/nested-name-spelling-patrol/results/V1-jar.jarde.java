// jarde: presentation of `V1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class V1 extends java.lang.Object {
    public V1() {
        // @method <init>()V
        // @declaration a constructor of `V1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int numSwitch(int arg0, int arg1) {
        // @method numSwitch(II)I
        // @declaration a static method of `V1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        switch (arg0) {
            case 1:
                return arg1 + 1;
            case 2:
                return arg1 * 2;
            case 3:
            case 4:
            default:
                return -arg1;
            case 5:
                return arg1 - 5;
        }
    }

    public static java.lang.String strSwitch(java.lang.String arg0) {
        // @method strSwitch(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `V1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        switch (arg0) {
            case "alpha":
                return "A";
            case "beta":
                return "B";
            case "gamma":
                return "G";
            default:
                return "?";
        }
    }

    public static int enumSwitch(V1$Op arg0, int arg1) {
        // @method enumSwitch(LV1$Op;I)I
        // @declaration a static method of `V1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        switch (arg0.ordinal()) {
            case 0:
                return arg1 + 1;
            case 1:
                return arg1 - 1;
            case 2:
                return arg1 * 2;
            default:
                return 0;
        }
    }

    public static int fallThrough(int arg0) {
        // @method fallThrough(I)I
        // @declaration a static method of `V1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        local1 = 0;
        switch (arg0) {
            case 1:
                local1 = local1 + 1;
            case 2:
                local1 = local1 + 2;
                break;
            case 3:
                local1 = local1 + 3;
                break;
            default:
                local1 = -1;
                break;
        }
        return local1;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `V1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("" + numSwitch(2, 10) + ":" + numSwitch(9, 10));
        java.lang.System.out.println(strSwitch("beta") + ":" + strSwitch("zz"));
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(enumSwitch(V1.Op.MUL, 7)).append(":").append(enumSwitch(V1.Op.ADD, 7)).toString());
        java.lang.System.out.println("" + fallThrough(1) + ":" + fallThrough(3));
        return;
    }

    enum Op {
        ADD,
        SUB,
        MUL;
    }
}
