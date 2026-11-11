// jarde: presentation of `Cf10Refused` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Cf10Refused extends java.lang.Object {
    public Cf10Refused() {
        // @method <init>()V
        // @declaration a constructor of `Cf10Refused`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int shared(int[] data) {
        // jarde: not recovered: the recovery run for `shared([I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method shared([I)I
        // @declaration a static method of `Cf10Refused`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 6 12 21 26 32
        // local 1 crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write
    }

    static int crossBlock(int[] data, boolean flag) {
        // jarde: not recovered: the recovery run for `crossBlock([IZ)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method crossBlock([IZ)I
        // @declaration a static method of `Cf10Refused`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 4 10 14 20 21 31 36 42
        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }

    static int intervalEffect(int[] data) {
        // jarde: not recovered: the recovery run for `intervalEffect([I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method intervalEffect([I)I
        // @declaration a static method of `Cf10Refused`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 6 12 25 30 36
        // local 1 crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Cf10Refused`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int[] d = new int[]{1, 2, 3, 4, 5};
        java.lang.System.out.println(shared(d));
        java.lang.System.out.println(crossBlock(d, true));
        java.lang.System.out.println(intervalEffect(d));
        return;
    }
}
