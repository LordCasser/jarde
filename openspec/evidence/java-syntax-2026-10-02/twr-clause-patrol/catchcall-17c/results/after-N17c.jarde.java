// jarde: presentation of `N17c` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class N17c extends java.lang.Object implements java.lang.AutoCloseable {
    java.lang.StringBuilder LOG;

    public N17c() {
        // @method <init>()V
        // @declaration a constructor of `N17c`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void close() {
        // @method close()V
        // @declaration an instance method of `N17c`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    private static void give() {
        // @method give()V
        // @declaration a static method of `N17c`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    public static java.lang.String popWrongValue() {
        // jarde: not recovered: the recovery run for `popWrongValue()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method popWrongValue()Ljava/lang/String;
        // @declaration a static method of `N17c`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 18 26 32 34 37 52
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String popSecondReader() {
        // jarde: not recovered: the recovery run for `popSecondReader()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method popSecondReader()Ljava/lang/String;
        // @declaration a static method of `N17c`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 18 26 32 34 37 52
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String popAfterCast() {
        // jarde: not recovered: the recovery run for `popAfterCast()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method popAfterCast()Ljava/lang/String;
        // @declaration a static method of `N17c`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 18 26 32 34 37 53
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `N17c`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) popWrongValue());
        java.lang.System.out.println((java.lang.String) popSecondReader());
        java.lang.System.out.println((java.lang.String) popAfterCast());
        return;
    }
}
