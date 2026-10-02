// jarde: presentation of `L3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class L3 extends java.lang.Object {
    public L3() {
        // @method <init>()V
        // @declaration a constructor of `L3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String tripleNoJump(int arg0) {
        // @method tripleNoJump(I)Ljava/lang/String;
        // @declaration a static method of `L3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        int local3;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            int local4;
            for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
                for (local4 = 0; local4 < arg0; local4 = local4 + 1) {
                    local1.append(local2).append(local3).append(local4).append(' ');
                }
            }
        }
        return local1.toString();
    }

    public static java.lang.String tripleBrkOnly(int arg0) {
        // @method tripleBrkOnly(I)Ljava/lang/String;
        // @declaration a static method of `L3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        int local3;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            int local4;
            for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
                local4 = 0;
                while (local4 < arg0) {
                    if (local4 == 1) {
                        break;
                    } else {
                        local1.append(local2).append(local3).append(local4).append(' ');
                        local4 = local4 + 1;
                    }
                }
            }
        }
        return local1.toString();
    }

    public static java.lang.String tripleContOnly(int arg0) {
        // @method tripleContOnly(I)Ljava/lang/String;
        // @declaration a static method of `L3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        int local3;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            int local4;
            for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
                for (local4 = 0; local4 < arg0; local4 = local4 + 1) {
                    if (local4 == 1) {
                    } else {
                        local1.append(local2).append(local3).append(local4).append(' ');
                    }
                }
            }
        }
        return local1.toString();
    }

    public static java.lang.String dblWhileDo() {
        // @method dblWhileDo()Ljava/lang/String;
        // @declaration a static method of `L3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local0;
        local0 = 0;
        while (local0 < 10) {
            local0 = local0 + 1;
            if (local0 == 5) {
            }
        }
        return "i=" + local0;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `L3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) tripleNoJump(2));
        java.lang.System.out.println((java.lang.String) tripleBrkOnly(2));
        java.lang.System.out.println((java.lang.String) tripleContOnly(2));
        java.lang.System.out.println((java.lang.String) dblWhileDo());
        return;
    }
}
