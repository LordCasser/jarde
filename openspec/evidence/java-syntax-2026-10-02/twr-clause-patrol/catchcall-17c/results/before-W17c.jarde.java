// jarde: presentation of `W17c` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class W17c extends java.lang.Object implements java.lang.AutoCloseable {
    static java.lang.StringBuilder log;

    static boolean closeBoom;

    public W17c() {
        // @method <init>()V
        // @declaration a constructor of `W17c`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void close() {
        // @method close()V
        // @declaration an instance method of `W17c`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        if (W17c.closeBoom) {
            throw new java.lang.IllegalStateException("close");
        } else {
            return;
        }
    }

    static void touch(W17c arg0) {
        // @method touch(LW17c;)V
        // @declaration a static method of `W17c`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    static void boom() {
        // @method boom()V
        // @declaration a static method of `W17c`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        throw new java.lang.IllegalStateException("body");
    }

    public static java.lang.String normalReturn() {
        // @method normalReturn()Ljava/lang/String;
        // @declaration a static method of `W17c`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        try (W17c local0 = new W17c()) {
            touch(local0);
        } catch (java.lang.IllegalStateException local0) {
            return "caught";
        }
        return "done";
    }

    public static java.lang.String callCatch() {
        // jarde: not recovered: the recovery run for `callCatch()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method callCatch()Ljava/lang/String;
        // @declaration a static method of `W17c`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 19 27 33 35 38 48
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String bodyThrows() {
        // jarde: not recovered: the recovery run for `bodyThrows()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method bodyThrows()Ljava/lang/String;
        // @declaration a static method of `W17c`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 18 26 32 34 37 47
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String callThenReturn() {
        // jarde: not recovered: the recovery run for `callThenReturn()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method callThenReturn()Ljava/lang/String;
        // @declaration a static method of `W17c`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 18 26 32 34 37 50
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String twoCalls() {
        // jarde: not recovered: the recovery run for `twoCalls()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method twoCalls()Ljava/lang/String;
        // @declaration a static method of `W17c`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 18 26 32 34 37 56
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String chainedConsume() {
        // jarde: not recovered: the recovery run for `chainedConsume()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method chainedConsume()Ljava/lang/String;
        // @declaration a static method of `W17c`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 18 26 32 34 37 49
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String branchBody() {
        // jarde: not recovered: the recovery run for `branchBody()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method branchBody()Ljava/lang/String;
        // @declaration a static method of `W17c`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 18 26 32 34 37 53 65 74
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String closeThrows() {
        // jarde: not recovered: the recovery run for `closeThrows()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method closeThrows()Ljava/lang/String;
        // @declaration a static method of `W17c`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 19 27 33 35 38 48
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String suppressedBoth() {
        // jarde: not recovered: the recovery run for `suppressedBoth()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method suppressedBoth()Ljava/lang/String;
        // @declaration a static method of `W17c`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 18 26 32 34 37 75
        // local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `W17c`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) normalReturn());
        java.lang.System.out.println((java.lang.String) callCatch());
        java.lang.System.out.println((java.lang.String) bodyThrows());
        java.lang.System.out.println((java.lang.String) callThenReturn());
        java.lang.System.out.println((java.lang.String) twoCalls());
        java.lang.System.out.println((java.lang.String) chainedConsume());
        W17c.closeBoom = true;
        java.lang.System.out.println((java.lang.String) closeThrows());
        java.lang.System.out.println((java.lang.String) suppressedBoth());
        W17c.closeBoom = false;
        java.lang.System.out.println((java.lang.String) branchBody());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("log:").append((java.lang.Object) W17c.log).toString());
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `W17c`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        W17c.log = new java.lang.StringBuilder();
        W17c.closeBoom = false;
    }
}
