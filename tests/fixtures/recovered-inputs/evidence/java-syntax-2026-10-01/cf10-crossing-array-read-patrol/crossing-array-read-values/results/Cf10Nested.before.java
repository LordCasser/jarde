// jarde: presentation of `Cf10Nested` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Cf10Nested extends java.lang.Object {
    public Cf10Nested() {
        // @method <init>()V
        // @declaration a constructor of `Cf10Nested`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int inLoop(int[] data) {
        // jarde: not recovered: the recovery run for `inLoop([I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method inLoop([I)I
        // @declaration a static method of `Cf10Nested`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 4 5 6 7 10 11 12 13 14 15 16 17 18 19 20 23 24 25 28 29 32 35 36 37 38 39 40 41 42 43 44 47 50 51
        // the graph is not reducible over 4 block(s) [4, 10, 32, 44]: a loop is entered at more than its header or two loops cross, which no Java structure spells
    }

    static int aroundLoop(int[] data) {
        // jarde: not recovered: the recovery run for `aroundLoop([I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method aroundLoop([I)I
        // @declaration a static method of `Cf10Nested`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 4 10 28 32 38 41 48
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    static int risky(int v) {
        // @method risky(I)I
        // @declaration a static method of `Cf10Nested`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        if (v == 3) {
            throw new java.lang.IllegalStateException("r");
        } else {
            return v;
        }
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Cf10Nested`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int[] d = new int[]{1, 2, 3, 4, 5};
        java.lang.System.out.println(inLoop(d));
        java.lang.System.out.println(aroundLoop(d));
        return;
    }
}
