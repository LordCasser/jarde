// jarde: presentation of `IO` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class IO extends java.lang.Object {
    public IO() {
        // @method <init>()V
        // @declaration a constructor of `IO`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int countLines(java.lang.String arg0) throws java.io.IOException {
        // jarde: not recovered: the recovery run for `countLines(Ljava/lang/String;)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method countLines(Ljava/lang/String;)I
        // @declaration a static method of `IO`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 27 36 42 52
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    static java.lang.String readAll(java.lang.String arg0) throws java.io.IOException {
        // jarde: not recovered: the recovery run for `readAll(Ljava/lang/String;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method readAll(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `IO`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 17 27 37 44 53
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `IO`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("" + countLines("data.txt") + "/" + readAll("data.txt").replace((java.lang.CharSequence) "\n", (java.lang.CharSequence) "|"));
        return;
    }
}
