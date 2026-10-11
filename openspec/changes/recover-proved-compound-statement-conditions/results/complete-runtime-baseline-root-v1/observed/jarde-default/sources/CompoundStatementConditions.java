// jarde: presentation of `CompoundStatementConditions` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class CompoundStatementConditions extends java.lang.Object {
    private CompoundStatementConditions() {
        // @method <init>()V
        // @declaration a constructor of `CompoundStatementConditions`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static boolean probe(java.lang.StringBuilder trace, java.lang.String label, boolean value) {
        // @method probe(Ljava/lang/StringBuilder;Ljava/lang/String;Z)Z
        // @declaration a static method of `CompoundStatementConditions`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        trace.append("test:").append(label).append('=').append(value).append(';');
        return value;
    }

    private static boolean probeOrThrow(java.lang.StringBuilder trace, java.lang.String label, boolean value, java.lang.String throwAt) {
        // @method probeOrThrow(Ljava/lang/StringBuilder;Ljava/lang/String;ZLjava/lang/String;)Z
        // @declaration a static method of `CompoundStatementConditions`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        trace.append("test:").append(label).append('=').append(value).append(';');
        if (label.equals((java.lang.Object) throwAt)) {
            throw new java.lang.IllegalStateException("probe-" + label);
        } else {
            return value;
        }
    }

    private static void appendLabel(java.lang.StringBuilder trace, java.lang.StringBuilder payload, java.lang.String label) {
        // @method appendLabel(Ljava/lang/StringBuilder;Ljava/lang/StringBuilder;Ljava/lang/String;)V
        // @declaration a static method of `CompoundStatementConditions`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        trace.append("emit:").append(label).append(';');
        if (payload.length() != 0) {
            payload.append('>');
        }
        payload.append(label);
        return;
    }

    public static java.lang.String test(boolean a, boolean b, boolean c, boolean d, java.lang.StringBuilder trace) {
        // jarde: not recovered: the recovery run for `test(ZZZZLjava/lang/StringBuilder;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method test(ZZZZLjava/lang/StringBuilder;)Ljava/lang/String;
        // @declaration a static method of `CompoundStatementConditions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 3 4 7 9 11 13 15 18 20 22 23 26 29 31 33 34 37 40 42 44 45 48 51 53 55 56 59 62 64 66 68 71 74 76 78 80 83 85 87 89 92 94 97
        // canonical block at BCI 62 on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted
    }

    public static java.lang.String testMixed(boolean a, boolean b, boolean c, boolean d, java.lang.StringBuilder trace) {
        // jarde: not recovered: the recovery run for `testMixed(ZZZZLjava/lang/StringBuilder;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method testMixed(ZZZZLjava/lang/StringBuilder;)Ljava/lang/String;
        // @declaration a static method of `CompoundStatementConditions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 3 4 7 9 11 13 15 18 20 22 23 26 29 31 33 34 37 40 42 44 45 48 51 53 55 56 59 62 64 66 68 71 74 76 78 80 83 85 87 89 92 94 97
        // canonical block at BCI 74 on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted
    }

    public static java.lang.String testThrowing(boolean a, boolean b, boolean c, boolean d, java.lang.String throwAt, java.lang.StringBuilder trace) {
        // jarde: not recovered: the recovery run for `testThrowing(ZZZZLjava/lang/String;Ljava/lang/StringBuilder;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method testThrowing(ZZZZLjava/lang/String;Ljava/lang/StringBuilder;)Ljava/lang/String;
        // @declaration a static method of `CompoundStatementConditions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 3 4 7 9 11 13 15 18 20 22 23 25 28 31 33 35 36 38 41 44 46 48 49 51 54 57 59 61 62 64 67 70 72 74 76 79 82 84 86 88 91 93 95 97 100 102 105
        // canonical block at BCI 70 on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted
    }

    public static java.lang.String testFloat(float value, java.lang.StringBuilder trace) {
        // jarde: not recovered: the recovery run for `testFloat(FLjava/lang/StringBuilder;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method testFloat(FLjava/lang/StringBuilder;)Ljava/lang/String;
        // @declaration a static method of `CompoundStatementConditions`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 3 4 7 8 9 10 12 15 16 17 18 21 22 24 25 28 29 30 32 35 38 39 40 42 45 46 47 49 52 53 56
        // canonical block at BCI 45 on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted
    }
}
