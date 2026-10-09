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
        // @method viaMixed()I
        // @declaration a static method of `V3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.util.ArrayList((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.Number[]{java.lang.Integer.valueOf(1), java.lang.Long.valueOf(2L), java.lang.Double.valueOf(0x1.8000000000000p1d)})).size();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `V3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(viaMixed());
        return;
    }
}
