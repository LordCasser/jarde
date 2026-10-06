// jarde: presentation of `OP2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class OP2 extends java.lang.Object {
    public OP2() {
        // @method <init>()V
        // @declaration a constructor of `OP2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int shl(int arg0) {
        // @method shl(I)I
        // @declaration a static method of `OP2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        arg0 = arg0 << 2;
        return arg0;
    }

    static int shr(int arg0) {
        // @method shr(I)I
        // @declaration a static method of `OP2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        arg0 = arg0 >> 1;
        return arg0;
    }

    static int ushr(int arg0) {
        // @method ushr(I)I
        // @declaration a static method of `OP2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        arg0 = arg0 >>> 1;
        return arg0;
    }

    static long lshl(long arg0) {
        // @method lshl(J)J
        // @declaration a static method of `OP2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        arg0 = arg0 << 3;
        return arg0;
    }

    static float nanf() {
        // @method nanf()F
        // @declaration a static method of `OP2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return 0x0.000000p-126f / 0x0.000000p-126f;
    }

    static double pinf() {
        // @method pinf()D
        // @declaration a static method of `OP2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return 0x1.0000000000000p0d / 0x0.0000000000000p-1022d;
    }

    static double ninf() {
        // @method ninf()D
        // @declaration a static method of `OP2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return -0x1.0000000000000p0d / 0x0.0000000000000p-1022d;
    }

    static float pzero() {
        // @method pzero()F
        // @declaration a static method of `OP2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return -0x0.000000p-126f;
    }

    static boolean condAssign(int arg0) {
        // @method condAssign(I)Z
        // @declaration a static method of `OP2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 + 1 > 0;
    }

    static int condAssignOld(int arg0) {
        // jarde: not recovered: the recovery run for `condAssignOld(I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method condAssignOld(I)I
        // @declaration a static method of `OP2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 3 4 7 8 11 12 15 16 17 18
        // the short-circuit chain from BCI 4 through 8 reaches a shared value consumer at BCI 16, but this slice has no SSA proof for that value; the complete region is quoted
    }

    public static void main(java.lang.String[] arg0) {
        // jarde: not recovered: the recovery run for `main([Ljava/lang/String;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `OP2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 3 6 7 10 12 15 16 19 22 24 27 29 32 35 37 40 42 45 48 50 53 56 59 62 64 67 70 73 75 78 81 84 86 89 92 95 97 100 103 104 105 108 109 112 113 116 118 121 122 125 128 130 133 134 137 140 143 146
        // the saved producer at BCI 0 has 3 consumers, so one local binding cannot prove its execution count
        // @bytecode 3 6 7
        // the saved producer at BCI 7 has no bounded final expression consumer
        // @bytecode 3 6 7 12
        // the saved producer at BCI 12 has no bounded final expression consumer
        // @bytecode 16
        // the saved producer at BCI 16 has no bounded final expression consumer
        // @bytecode 3 6 7 12 16 19
        // the saved producer at BCI 19 has no bounded final expression consumer
        // @bytecode 3 6 7 12 16 19 24
        // the saved producer at BCI 24 has no bounded final expression consumer
        // @bytecode 29
        // the saved producer at BCI 29 has no bounded final expression consumer
        // @bytecode 3 6 7 12 16 19 24 29 32
        // the saved producer at BCI 32 has no bounded final expression consumer
        // @bytecode 3 6 7 12 16 19 24 29 32 37
        // the saved producer at BCI 37 has no bounded final expression consumer
        // @bytecode 42
        // the saved producer at BCI 42 has no bounded final expression consumer
        // @bytecode 3 6 7 12 16 19 24 29 32 37 42 45
        // the saved producer at BCI 45 has no bounded final expression consumer
        // @bytecode 3 6 7 12 16 19 24 29 32 37 42 45 50
        // the saved producer at BCI 50 has no bounded final expression consumer
        // @bytecode 56
        // the saved producer at BCI 56 has no bounded final expression consumer
        // @bytecode 3 6 7 12 16 19 24 29 32 37 42 45 50 56 59
        // the saved producer at BCI 59 has no bounded final expression consumer
        // @bytecode 3 6 7 12 16 19 24 29 32 37 42 45 50 56 59 64
        // the saved producer at BCI 64 has no bounded final expression consumer
        // @bytecode 67
        // the saved producer at BCI 67 has no bounded final expression consumer
        // @bytecode 3 6 7 12 16 19 24 29 32 37 42 45 50 56 59 64 67 70
        // the saved producer at BCI 70 has no bounded final expression consumer
        // @bytecode 3 6 7 12 16 19 24 29 32 37 42 45 50 56 59 64 67 70 75
        // the saved producer at BCI 75 has no bounded final expression consumer
        // @bytecode 78
        // the saved producer at BCI 78 has no bounded final expression consumer
        // @bytecode 3 6 7 12 16 19 24 29 32 37 42 45 50 56 59 64 67 70 75 78 81
        // the saved producer at BCI 81 has no bounded final expression consumer
        // @bytecode 3 6 7 12 16 19 24 29 32 37 42 45 50 56 59 64 67 70 75 78 81 86
        // the saved producer at BCI 86 has no bounded final expression consumer
        // @bytecode 89
        // the saved producer at BCI 89 has no bounded final expression consumer
        // @bytecode 3 6 7 12 16 19 24 29 32 37 42 45 50 56 59 64 67 70 75 78 81 86 89 92
        // the saved producer at BCI 92 has no bounded final expression consumer
        // @bytecode 3 6 7 12 16 19 24 29 32 37 42 45 50 56 59 64 67 70 75 78 81 86 89 92 97
        // the saved producer at BCI 97 has 3 consumers, so one local binding cannot prove its execution count
        // @bytecode 143 0 140 137 130 125 118 113 97 92 86 81 75 70 64 59 50 45 37 32 24 19 12 7 3 6 16 29 42 56 67 78 89 122 134
        // the value at BCI 143 was produced by a saved declaration this run could not commit
        jarde_refused_body();
    }
}
