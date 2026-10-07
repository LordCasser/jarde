// jarde: presentation of `ScvConcatReReadControls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ScvConcatReReadControls extends java.lang.Object {
    static boolean[] flags = new boolean[2];

    public ScvConcatReReadControls() {
        // @method <init>()V
        // @declaration a constructor of `ScvConcatReReadControls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String reRead(int arg0) {
        // jarde: not recovered: the recovery run for `reRead(I)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method reRead(I)Ljava/lang/String;
        // @declaration a static method of `ScvConcatReReadControls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 6 7 8 9 12 13 16 17 18 21 22 23 24 27 28 31 32 35 37 40 43
        // the short-circuit chain from BCI 3 through 9 reaches a shared value consumer at BCI 17, but this slice has no SSA proof for that value; the complete region is quoted
    }
}
