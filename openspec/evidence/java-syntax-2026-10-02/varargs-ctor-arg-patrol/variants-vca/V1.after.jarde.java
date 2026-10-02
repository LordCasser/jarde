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
        // @method viaEmptyCall()I
        // @declaration a static method of `V1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.util.ArrayList((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.String[0])).size();
    }

    public static int viaBoxedMix() {
        // @method viaBoxedMix()I
        // @declaration a static method of `V1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.util.ArrayList((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.Integer[]{java.lang.Integer.valueOf(1), java.lang.Integer.valueOf(2), java.lang.Integer.valueOf(3)})).size();
    }

    public static int viaHashSet() {
        // @method viaHashSet()I
        // @declaration a static method of `V1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.util.HashSet((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.String[]{"a", "b"})).size();
    }

    public static int viaCallElements() {
        // @method viaCallElements()I
        // @declaration a static method of `V1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.util.ArrayList((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.Integer[]{java.lang.Integer.valueOf(7), box(8)})).size();
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
