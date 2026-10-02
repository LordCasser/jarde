// jarde: presentation of `L4` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class L4 extends java.lang.Object {
    public L4() {
        // @method <init>()V
        // @declaration a constructor of `L4`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String contMidPlain(int arg0) {
        // @method contMidPlain(I)Ljava/lang/String;
        // @declaration a static method of `L4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        int local3;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            int local4;
            for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
                for (local4 = 0; local4 < arg0; local4 = local4 + 1) {
                    if (local3 == 2) {
                    } else {
                        local1.append(local2).append(local3).append(local4).append(' ');
                    }
                }
            }
        }
        return local1.toString();
    }

    public static java.lang.String contMidLabel(int arg0) {
        // @method contMidLabel(I)Ljava/lang/String;
        // @declaration a static method of `L4`, member flags 0x0009
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
                    if (local3 == 2) {
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

    public static java.lang.String brkMidPlain(int arg0) {
        // @method brkMidPlain(I)Ljava/lang/String;
        // @declaration a static method of `L4`, member flags 0x0009
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
                    if (local3 == 2) {
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

    public static java.lang.String brkOuterFromDeep(int arg0) {
        // @method brkOuterFromDeep(I)Ljava/lang/String;
        // @declaration a static method of `L4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        int local3;
        loop: for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            int local4;
            for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
                local4 = 0;
                while (local4 < arg0) {
                    if (local2 == 2) {
                        break loop;
                    } else {
                        local1.append(local2).append(local3).append(local4).append(' ');
                        local4 = local4 + 1;
                    }
                }
            }
        }
        return local1.toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `L4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) contMidPlain(3));
        java.lang.System.out.println((java.lang.String) contMidLabel(3));
        java.lang.System.out.println((java.lang.String) brkMidPlain(3));
        java.lang.System.out.println((java.lang.String) brkOuterFromDeep(3));
        return;
    }
}
