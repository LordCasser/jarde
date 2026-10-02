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
        // jarde: not recovered: the recovery run for `deepLabels(I)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method deepLabels(I)Ljava/lang/String;
        // @declaration a static method of `L1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 10 15 17 22 27 30 35 38 41 47 53 56 62 65 85 91 97 103
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String labelWhile() {
        // jarde: not recovered: the recovery run for `labelWhile()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method labelWhile()Ljava/lang/String;
        // @declaration a static method of `L1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 5 8 11 12 13 14 17 20 21 23 26 29 32 33 36 38 41 42 45 48
        // canonical block at BCI 2 on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted
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
