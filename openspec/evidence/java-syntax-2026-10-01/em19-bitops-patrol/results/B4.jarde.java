// jarde: presentation of `B4` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class B4 extends java.lang.Object {
    public B4() {
        // @method <init>()V
        // @declaration a constructor of `B4`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String v1(int arg0) {
        // jarde: not recovered: the recovery run for `v1(I)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method v1(I)Ljava/lang/String;
        // @declaration a static method of `B4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 5 6 7 8 10 11 12 13 15 16 17 18 19 20 21 22 23 24 25 26 27 28 31 32 33 34 37 38 41 42 43 46 47 50 51 54 56 59 60 63 66
        // the short-circuit chain from BCI 28 through 34 reaches a shared value consumer at BCI 42, but this slice has no SSA proof for that value; the complete region is quoted
    }

    public static java.lang.String v2(int arg0) {
        // jarde: not recovered: the recovery run for `v2(I)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method v2(I)Ljava/lang/String;
        // @declaration a static method of `B4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 5 6 7 8 9 10 13 14 15 16 19 20 23 24 25 26 27 28 31 32 34 35 38 39 42 43 44 47 48 51 52 55 57 60 61 64 66 69 70 73 76
        // the arms of the branch in block 13 do not meet at one join
    }

    public static java.lang.String v3(int arg0) {
        // jarde: not recovered: the recovery run for `v3(I)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method v3(I)Ljava/lang/String;
        // @declaration a static method of `B4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 6 7 8 9 12 13 16 17 18 19 20 21 24 25 27 28 31 32 35 36 37 40 41 44 45 48 50 53 54 57 60
        // the arms of the branch in block 6 do not meet at one join
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `B4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) v1(27));
        java.lang.System.out.println((java.lang.String) v2(27));
        java.lang.System.out.println((java.lang.String) v3(27));
        return;
    }
}
