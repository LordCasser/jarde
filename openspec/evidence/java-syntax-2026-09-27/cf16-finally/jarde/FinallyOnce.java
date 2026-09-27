// jarde: presentation of `FinallyOnce` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class FinallyOnce extends java.lang.Object {
    private static int cleanupCount;

    public FinallyOnce() {
        // @method <init>()V
        // @declaration a constructor of `FinallyOnce`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String handled(boolean fail) {
        // jarde: not recovered: the recovery run for `handled(Z)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method handled(Z)Ljava/lang/String;
        // @declaration a static method of `FinallyOnce`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 8 18 31 65
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static void escaping() {
        // jarde: not recovered: the recovery run for `escaping()V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method escaping()V
        // @declaration a static method of `FinallyOnce`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 4 7 8 10 13
        // BCI 14: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
        // @bytecode 14 15 18 19 20 23 24
        // 1 live block(s) are reachable only through edges the normal-flow view leaves out: [14]
    }

    public static int count() {
        // @method count()I
        // @declaration a static method of `FinallyOnce`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return FinallyOnce.cleanupCount;
    }

    public static void main(java.lang.String[] args) {
        // jarde: not recovered: the recovery run for `main([Ljava/lang/String;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `FinallyOnce`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 3 6 7 10 11 14 17 19 22 25 28 31 34 37 40 41 44 45 48 51 53 56 59 62 65 68 71 74 76 79
        // BCI 65: the resource's own initialisation is not one statement of this block whose value lands in a slot: writing it in the header would move or drop an effect
        // @bytecode 82 83 86 89 90 93 94 97 100 102 105 108 111 114 117
        // 2 live block(s) are reachable only through edges the normal-flow view leaves out: [117, 82]
    }
}
