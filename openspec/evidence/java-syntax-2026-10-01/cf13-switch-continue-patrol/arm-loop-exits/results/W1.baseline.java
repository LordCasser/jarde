// jarde: presentation of `W1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class W1 extends java.lang.Object {
    public W1() {
        // @method <init>()V
        // @declaration a constructor of `W1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int mix(int arg0) {
        // jarde: not recovered: the recovery run for `mix(I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method mix(I)I
        // @declaration a static method of `W1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 1 2 3 4 5 6 9 10 11 12 40 43 46 49 52 53 54 57 60 63 66 69 72 73
        // canonical block at BCI 9 on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `W1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(mix(7));
        java.lang.System.out.println(mix(3));
        return;
    }
}
