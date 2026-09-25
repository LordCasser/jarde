// jarde: presentation of `IterableExceptionProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
final class IterableExceptionProbe extends java.lang.Object {
    static java.lang.String log;

    static int caught;

    static int finalized;

    IterableExceptionProbe() {
        // @method <init>()V
        // @declaration a constructor of `IterableExceptionProbe`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int uniform(java.lang.Iterable arg0) {
        // jarde: not recovered: the recovery run for `uniform(Ljava/lang/Iterable;)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method uniform(Ljava/lang/Iterable;)I
        // @declaration a static method of `IterableExceptionProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 9 18 38 73 141 178
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    static int iteratorBoundary(java.lang.Iterable arg0) {
        // jarde: not recovered: the recovery run for `iteratorBoundary(Ljava/lang/Iterable;)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method iteratorBoundary(Ljava/lang/Iterable;)I
        // @declaration a static method of `IterableExceptionProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 34 69 71 80 100 135 203 240
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    static int nextBoundary(java.lang.Iterable arg0) {
        // jarde: not recovered: the recovery run for `nextBoundary(Ljava/lang/Iterable;)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method nextBoundary(Ljava/lang/Iterable;)I
        // @declaration a static method of `IterableExceptionProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 33 42 62 98 133 170
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `IterableExceptionProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `IterableExceptionProbe`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        IterableExceptionProbe.log = "";
    }
}
