// jarde: presentation of `L2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class L2 extends java.lang.Object {
    public L2() {
        // @method <init>()V
        // @declaration a constructor of `L2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String contOuter(int arg0) {
        // @method contOuter(I)Ljava/lang/String;
        // @declaration a static method of `L2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        int local3;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            local3 = 0;
            while (local3 < arg0) {
                if (local3 == 2) {
                    break;
                } else {
                    local1.append(local2).append(local3).append(' ');
                    local3 = local3 + 1;
                }
            }
        }
        return local1.toString();
    }

    public static java.lang.String brkOuter(int arg0) {
        // @method brkOuter(I)Ljava/lang/String;
        // @declaration a static method of `L2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        int local3;
        loop: for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            local3 = 0;
            while (local3 < arg0) {
                if (local2 == 2) {
                    break loop;
                } else {
                    local1.append(local2).append(local3).append(' ');
                    local3 = local3 + 1;
                }
            }
        }
        return local1.toString();
    }

    public static java.lang.String triplePlain(int arg0) {
        // jarde: not recovered: the recovery run for `triplePlain(I)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method triplePlain(I)Ljava/lang/String;
        // @declaration a static method of `L2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 10 15 17 22 25 31 37 40 45 48 68 74 80 86
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String innerLabel(int arg0) {
        // @method innerLabel(I)Ljava/lang/String;
        // @declaration a static method of `L2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        int local3;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            for (local3 = 0; local3 < arg0; local3 = local3 + 1) {
                if (local3 == 1) {
                } else {
                    local1.append(local2).append(local3).append(' ');
                }
            }
        }
        return local1.toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `L2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) contOuter(3));
        java.lang.System.out.println((java.lang.String) brkOuter(3));
        java.lang.System.out.println((java.lang.String) triplePlain(3));
        java.lang.System.out.println((java.lang.String) innerLabel(3));
        return;
    }
}
