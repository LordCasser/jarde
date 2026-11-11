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

    public static java.lang.String labeled() {
        // @method labeled()Ljava/lang/String;
        // @declaration a static method of `L1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local0;
        int local1;
        local0 = new java.lang.StringBuilder();
        int local2;
        jarde_loop_10: for (local1 = 0; local1 < 3; local1 = local1 + 1) {
            local2 = 0;
            while (local2 < 3) {
                if (local2 == 1) {
                    break;
                } else if (local1 == 2) {
                    break jarde_loop_10;
    } else {
                    local0.append(local1).append(local2).append(',');
                    local2 = local2 + 1;
    }
            }
        }
        return local0.toString();
    }

    public static java.lang.String plainNested() {
        // @method plainNested()Ljava/lang/String;
        // @declaration a static method of `L1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local0;
        int local1;
        local0 = new java.lang.StringBuilder();
        int local2;
        for (local1 = 0; local1 < 2; local1 = local1 + 1) {
            local2 = 0;
            while (local2 < 2) {
                if (local2 == 1) {
                    break;
                } else {
                    local0.append(local1).append(local2);
                    local2 = local2 + 1;
                }
            }
        }
        return local0.toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `L1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) labeled());
        java.lang.System.out.println((java.lang.String) plainNested());
        return;
    }
}
