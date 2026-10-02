// jarde: presentation of `V1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class V1 extends java.lang.Object {
    public V1() {
        // @method <init>()V
        // @declaration a constructor of `V1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int viaEmptyCall() {
        // jarde: not recovered: the recovery run for `viaEmptyCall()I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method viaEmptyCall()I
        // @declaration a static method of `V1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 17 14 11 8 5
        // the copy at BCI 3 has no proved local assignment
    }

    public static int viaBoxedMix() {
        // jarde: not recovered: the recovery run for `viaBoxedMix()I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method viaBoxedMix()I
        // @declaration a static method of `V1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 38 35 32 29 5 4 8 9 10 11 14 15 16 17 18 21 22 23 24 25 28
        // the copy at BCI 3 has no proved local assignment
    }

    public static int viaHashSet() {
        // jarde: not recovered: the recovery run for `viaHashSet()I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method viaHashSet()I
        // @declaration a static method of `V1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 27 24 21 18 5 4 8 9 10 12 13 14 15 17
        // the copy at BCI 3 has no proved local assignment
    }

    public static int viaCallElements() {
        // jarde: not recovered: the recovery run for `viaCallElements()I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method viaCallElements()I
        // @declaration a static method of `V1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 33 30 27 24 5 4 8 9 10 12 15 16 17 18 20 23
        // the copy at BCI 3 has no proved local assignment
    }

    static java.lang.Integer box(int arg0) {
        // @method box(I)Ljava/lang/Integer;
        // @declaration a static method of `V1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.Integer.valueOf(arg0);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `V1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("" + viaEmptyCall() + ":" + viaBoxedMix() + ":" + viaHashSet() + ":" + viaCallElements());
        return;
    }
}
