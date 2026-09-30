// jarde: presentation of `Cf10Values` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Cf10Values extends java.lang.Object {
    public Cf10Values() {
        // @method <init>()V
        // @declaration a constructor of `Cf10Values`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int intSum(int[] data) {
        // jarde: not recovered: the recovery run for `intSum([I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method intSum([I)I
        // @declaration a static method of `Cf10Values`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 4 10 28 32 38
        // local 1 crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write
    }

    public static double doubleSum(double[] data) {
        // jarde: not recovered: the recovery run for `doubleSum([D)D` produced no statement (explanation only); the artifact's own comment lines are below
        // @method doubleSum([D)D
        // @declaration a static method of `Cf10Values`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 4 10 25 31 37
        // local 1 crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write
    }

    public static java.lang.String lastRef(java.lang.String[] parts) {
        // jarde: not recovered: the recovery run for `lastRef([Ljava/lang/String;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method lastRef([Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `Cf10Values`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 6 12 23 28 34
        // local 1 crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write
    }

    public static int catchMultiWrite(int[] data) {
        // jarde: not recovered: the recovery run for `catchMultiWrite([I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method catchMultiWrite([I)I
        // @declaration a static method of `Cf10Values`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 4 10 28 39 45
        // local 1 crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write
    }

    public static int nestedTry(int[] data) {
        // jarde: not recovered: the recovery run for `nestedTry([I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method nestedTry([I)I
        // @declaration a static method of `Cf10Values`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 4 5 6 7 10 11 12 13 14 15 16 17 18 19 20 23 24 25 28 29 32 35 36 37 38 39 40 41 42 43 44 47 50 51
        // the graph is not reducible over 4 block(s) [4, 10, 32, 44]: a loop is entered at more than its header or two loops cross, which no Java structure spells
    }

    static int risky(int v) {
        // @method risky(I)I
        // @declaration a static method of `Cf10Values`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (v == 3) {
            throw new java.lang.IllegalStateException("r");
        } else {
            return v;
        }
    }

    static void noise(double v) {
        // @method noise(D)V
        // @declaration a static method of `Cf10Values`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (v == 0x1.8000000000000p1d) {
            throw new java.lang.IllegalStateException("n");
        } else {
            return;
        }
    }

    static void noise(java.lang.String v) {
        // @method noise(Ljava/lang/String;)V
        // @declaration a static method of `Cf10Values`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if ("boom".equals((java.lang.Object) v)) {
            throw new java.lang.IllegalStateException("n");
        } else {
            return;
        }
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Cf10Values`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int[] d = new int[]{1, 2, 3, 4, 5};
        java.lang.System.out.println(intSum(d));
        double[] f = new double[]{0x1.8000000000000p0d, 0x1.4000000000000p1d, 0x1.8000000000000p1d, 0x1.0000000000000p2d, 0x1.6000000000000p2d};
        java.lang.System.out.println(doubleSum(f));
        java.lang.String[] p = new java.lang.String[]{"a", "boom", "c", "d", "e", "f"};
        java.lang.System.out.println((java.lang.String) lastRef(p));
        java.lang.System.out.println(catchMultiWrite(d));
        java.lang.System.out.println(nestedTry(d));
        return;
    }
}
