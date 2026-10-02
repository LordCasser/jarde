// jarde: presentation of `W4` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class W4 extends java.lang.Object {
    public W4() {
        // @method <init>()V
        // @declaration a constructor of `W4`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int bare() {
        // @method bare()I
        // @declaration a static method of `W4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return java.util.Arrays.asList((java.lang.Object[]) new java.lang.Integer[]{java.lang.Integer.valueOf(1), java.lang.Integer.valueOf(2)}).size();
    }

    public static java.lang.String fmt() {
        // @method fmt()Ljava/lang/String;
        // @declaration a static method of `W4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return java.lang.String.format("%d-%d", new java.lang.Object[]{java.lang.Integer.valueOf(1), java.lang.Integer.valueOf(2)});
    }

    public static int assigned() {
        // @method assigned()I
        // @declaration a static method of `W4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.util.List local0 = java.util.Arrays.asList((java.lang.Object[]) new java.lang.Integer[]{java.lang.Integer.valueOf(5), java.lang.Integer.valueOf(6)});
        return ((java.lang.Integer) local0.get(0)).intValue();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `W4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("" + bare() + ":" + fmt() + ":" + assigned());
        return;
    }
}
