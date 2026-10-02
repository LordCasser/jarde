// jarde: presentation of `L5` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class L5 extends java.lang.Object {
    public L5() {
        // @method <init>()V
        // @declaration a constructor of `L5`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String brkSelfContSelf(int arg0) {
        // @method brkSelfContSelf(I)Ljava/lang/String;
        // @declaration a static method of `L5`, member flags 0x0009
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
                        break;
                    } else if (local4 == 0) {
    } else {
                        local1.append(local2).append(local3).append(local4).append(' ');
    }
                }
            }
        }
        return local1.toString();
    }

    public static java.lang.String brkSelfContMid(int arg0) {
        // @method brkSelfContMid(I)Ljava/lang/String;
        // @declaration a static method of `L5`, member flags 0x0009
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
                        break;
                    } else if (local3 == 2) {
    } else {
                        local1.append(local2).append(local3).append(local4).append(' ');
    }
                }
            }
        }
        return local1.toString();
    }

    public static java.lang.String dblJumpDoWhile() {
        // @method dblJumpDoWhile()Ljava/lang/String;
        // @declaration a static method of `L5`, member flags 0x0009
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

    public static java.lang.String dblJumpDoWhilePlain() {
        // @method dblJumpDoWhilePlain()Ljava/lang/String;
        // @declaration a static method of `L5`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local0;
        local0 = 0;
        while (local0 < 10) {
            local0 = local0 + 1;
            if (local0 % 3 == 0) {
            } else if (local0 > 7) {
    }
        }
        return "i=" + local0;
        // @bytecode 26 28
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [26]
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `L5`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) brkSelfContSelf(2));
        java.lang.System.out.println((java.lang.String) brkSelfContMid(3));
        java.lang.System.out.println((java.lang.String) dblJumpDoWhile());
        java.lang.System.out.println((java.lang.String) dblJumpDoWhilePlain());
        return;
    }
}
