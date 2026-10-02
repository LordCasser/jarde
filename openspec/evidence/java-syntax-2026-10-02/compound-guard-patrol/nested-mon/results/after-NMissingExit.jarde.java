// jarde: presentation of `NMiss` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class NMiss extends java.lang.Object {
    static java.lang.StringBuilder log = new java.lang.StringBuilder();

    public NMiss() {
        // @method <init>()V
        // @declaration a constructor of `NMiss`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int m(int arg0) {
        // jarde: not recovered: the recovery run for `m(I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method m(I)I
        // @declaration a static method of `NMiss`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 16 22 33 38 45 49
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `NMiss`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("loaded " + m(3));
        return;
    }
}
