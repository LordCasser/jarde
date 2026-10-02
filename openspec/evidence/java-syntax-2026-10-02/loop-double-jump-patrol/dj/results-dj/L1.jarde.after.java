// jarde: presentation of `L1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class L1 extends java.lang.Object {
    public L1() {
        // @method <init>()V
        // @declaration a constructor of `L1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String deepLabels(int arg0) {
        // @method deepLabels(I)Ljava/lang/String;
        // @declaration a static method of `L1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        int local3;
        loop: for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            int local4;
            for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
                if (local3 == 2) {
                    break;
                }
                if (local2 == 3) {
                    break loop;
                }
                for (local4 = 0; local4 < arg0; local4 = local4 + 1) {
                    if (local4 == 1) {
                    } else if (local4 == 2) {
                        break;
    } else {
                        local1.append(local2).append(local3).append(local4).append(' ');
    }
                }
            }
        }
        return local1.toString();
    }

    public static java.lang.String labelWhile() {
        // @method labelWhile()Ljava/lang/String;
        // @declaration a static method of `L1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local0;
        local0 = 0;
        while (local0 < 10) {
            local0 = local0 + 1;
            if (local0 % 3 == 0) {
            } else if (local0 > 7) {
                break;
    }
        }
        return "i=" + local0;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `L1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) deepLabels(5));
        java.lang.System.out.println((java.lang.String) labelWhile());
        return;
    }
}
