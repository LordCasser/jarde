// jarde: presentation of `BranchReads` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class BranchReads extends java.lang.Object {
    private BranchReads() {
        // @method <init>()V
        // @declaration a constructor of `BranchReads`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int ternaryRead(int x) {
        // @method ternaryRead(I)I
        // @declaration a static method of `BranchReads`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        x = x + 1;
        boolean b = x > 0 && x > 0;
        return b ? x : -1;
    }

    static int ifStatement(int x) {
        // @method ifStatement(I)I
        // @declaration a static method of `BranchReads`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        x = x + 1;
        boolean b = x > 0 && x > 0;
        if (b) {
            return x;
        } else {
            return -1;
        }
    }

    static boolean midChain(int x) {
        // @method midChain(I)Z
        // @declaration a static method of `BranchReads`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        x = x + 1;
        boolean b = x > 0 && x > 0;
        return x > 0 && (b && x < 100);
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `BranchReads`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(ternaryRead(0));
        java.lang.System.out.println(ternaryRead(-2));
        java.lang.System.out.println(ifStatement(0));
        java.lang.System.out.println(ifStatement(-2));
        java.lang.System.out.println(midChain(0));
        java.lang.System.out.println(midChain(-2));
        return;
    }
}
