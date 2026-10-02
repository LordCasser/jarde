// jarde: presentation of `NM` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class NM extends java.lang.Object {
    static java.lang.StringBuilder log = new java.lang.StringBuilder();

    public NM() {
        // @method <init>()V
        // @declaration a constructor of `NM`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int v1(int arg0) {
        // jarde: not recovered: the recovery run for `v1(I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method v1(I)I
        // @declaration a static method of `NM`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 16 22 33 38 45 49
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static int v2(int arg0) {
        // jarde: not recovered: the recovery run for `v2(I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method v2(I)I
        // @declaration a static method of `NM`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 9 14 31 39 45 49
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static int v3(int arg0) {
        // jarde: not recovered: the recovery run for `v3(I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method v3(I)I
        // @declaration a static method of `NM`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 16 22 33 39 46
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static int n3(int arg0) {
        // jarde: not recovered: the recovery run for `n3(I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method n3(I)I
        // @declaration a static method of `NM`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 34 42 47 54 58
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static int sr(java.lang.Object arg0, int arg1) {
        // jarde: not recovered: the recovery run for `sr(Ljava/lang/Object;I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method sr(Ljava/lang/Object;I)I
        // @declaration a static method of `NM`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 14 20 31 37 45 49
        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `NM`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(v1(5));
        java.lang.System.out.println(v2(4));
        java.lang.System.out.println(v3(5));
        java.lang.System.out.println(n3(5));
        java.lang.System.out.println(sr(new java.lang.Object(), 5));
        return;
    }
}
