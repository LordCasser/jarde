// jarde: presentation of `V3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class V3 extends java.lang.Object {
    public V3() {
        // @method <init>()V
        // @declaration a constructor of `V3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int viaMixed() {
        // jarde: not recovered: the recovery run for `viaMixed()I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method viaMixed()I
        // @declaration a static method of `V3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 42 39 33 5 4 8 9 10 11 14 15 16 17 20 23 24 25 26 29 32
        // the array initializer element at BCI 42 is presented as `java.lang.Integer`, while the array component is `java.lang.Number`; without a hierarchy fact this `aastore` cannot be rewritten as a Java initializer without changing its compatibility or exception behavior
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `V3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(viaMixed());
        return;
    }
}
