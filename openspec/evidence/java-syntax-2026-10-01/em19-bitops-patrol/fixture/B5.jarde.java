// jarde: presentation of `B5` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class B5 extends java.lang.Object {
    public B5() {
        // @method <init>()V
        // @declaration a constructor of `B5`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String s1(int arg0) {
        // jarde: not recovered: the recovery run for `s1(I)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method s1(I)Ljava/lang/String;
        // @declaration a static method of `B5`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 6 7 8 9 12 13 16 17 18 21 22 25 26 29 31 34 37
        // the short-circuit chain from BCI 3 through 9 reaches a shared value consumer at BCI 17, but this slice has no SSA proof for that value; the complete region is quoted
    }

    public static java.lang.String s2(int arg0) {
        // jarde: not recovered: the recovery run for `s2(I)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method s2(I)Ljava/lang/String;
        // @declaration a static method of `B5`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 6 7 8 9 12 13 16 17 18 19 20 21 24 25 28 29 30 33 34 37 38 41 43 46 47 50 53
        // the arms of the branch in block 6 do not meet at one join
    }

    public static boolean s3(int arg0) {
        // @method s3(I)Z
        // @declaration a static method of `B5`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        boolean local1 = (arg0 & 1) != 0 && (arg0 & 2) != 0;
        return local1;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `B5`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) s1(3));
        java.lang.System.out.println((java.lang.String) s2(3));
        java.lang.System.out.println(s3(3));
        return;
    }
}
