// jarde: presentation of `DWVariants` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class DWVariants extends java.lang.Object {
    public DWVariants() {
        // @method <init>()V
        // @declaration a constructor of `DWVariants`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String throwInDoWhile() {
        // jarde: not recovered: the recovery run for `throwInDoWhile()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method throwInDoWhile()Ljava/lang/String;
        // @declaration a static method of `DWVariants`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 2 8 17 20 26 53
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String doubleReturnDoWhile() {
        // jarde: not recovered: the recovery run for `doubleReturnDoWhile()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method doubleReturnDoWhile()Ljava/lang/String;
        // @declaration a static method of `DWVariants`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 2 8 17 20 26 46 52 72
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String sharedLeafDoWhile() {
        // jarde: not recovered: the recovery run for `sharedLeafDoWhile()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method sharedLeafDoWhile()Ljava/lang/String;
        // @declaration a static method of `DWVariants`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 5 8 11 12 13 14 17 20 21 23 26 27 29 32 35 36 39 41 44 45 48 51 52 55 56 59 61 64 65 68 71
        // the arms of the branch in block 26 do not meet at one join
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `DWVariants`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try {
            java.lang.System.out.println((java.lang.String) throwInDoWhile());
        } catch (java.lang.IllegalStateException local1) {
            java.lang.System.out.println((java.lang.String) local1.getMessage());
        }
        java.lang.System.out.println((java.lang.String) doubleReturnDoWhile());
        java.lang.System.out.println((java.lang.String) sharedLeafDoWhile());
        return;
    }
}
