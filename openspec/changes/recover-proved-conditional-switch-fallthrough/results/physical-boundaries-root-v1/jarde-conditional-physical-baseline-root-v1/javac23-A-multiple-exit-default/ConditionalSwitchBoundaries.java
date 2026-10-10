// jarde: presentation of `ConditionalSwitchBoundaries` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ConditionalSwitchBoundaries extends java.lang.Object {
    private ConditionalSwitchBoundaries() {
        // @method <init>()V
        // @declaration a constructor of `ConditionalSwitchBoundaries`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String partialBreak(int arg0, int arg1) {
        // jarde: not recovered: the recovery run for `partialBreak(II)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method partialBreak(II)Ljava/lang/String;
        // @declaration a static method of `ConditionalSwitchBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 36 40 50 51 53 56 57 67 74
        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String innerLoopBreak(int arg0, int arg1) {
        // jarde: not recovered: the recovery run for `innerLoopBreak(II)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method innerLoopBreak(II)Ljava/lang/String;
        // @declaration a static method of `ConditionalSwitchBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 36 38 43 54 57 63 70 80 87
        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    public static java.lang.String innerSwitchBreak(int arg0, int arg1) {
        // jarde: not recovered: the recovery run for `innerSwitchBreak(II)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method innerSwitchBreak(II)Ljava/lang/String;
        // @declaration a static method of `ConditionalSwitchBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 3 4 7 8 9 36 37 56 57 59 62 63 66 67 69 72 73 74 76 79 80 81 83 86 87 90 91 93 96 97 98 101
        // the arms of the branch in block 0 do not meet at one join
    }

    public static java.lang.String terminalCase(int arg0) {
        // @method terminalCase(I)Ljava/lang/String;
        // @declaration a static method of `ConditionalSwitchBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        local1 = new java.lang.StringBuilder();
        switch (arg0) {
            case 0:
                local1.append("R");
                return local1.toString();
            case 1:
                local1.append("T");
                throw new java.lang.IllegalArgumentException((java.lang.String) local1.toString());
            case 2:
                local1.append("C");
                break;
            default:
                local1.append("D");
                break;
        }
        return local1.toString();
    }

    public static java.lang.String caughtExceptionThenFallthrough(int arg0, int arg1) {
        // jarde: not recovered: the recovery run for `caughtExceptionThenFallthrough(II)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method caughtExceptionThenFallthrough(II)Ljava/lang/String;
        // @declaration a static method of `ConditionalSwitchBoundaries`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 36 40 45 47 60 68 78 85
        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }
}
