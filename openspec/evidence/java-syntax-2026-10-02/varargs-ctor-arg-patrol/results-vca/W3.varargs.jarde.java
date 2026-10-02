// jarde: presentation of `W3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class W3 extends java.lang.Object {
    public W3() {
        // @method <init>()V
        // @declaration a constructor of `W3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int viaArrays() {
        // @method viaArrays()I
        // @declaration a static method of `W3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.util.ArrayList((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.Integer[]{java.lang.Integer.valueOf(1), java.lang.Integer.valueOf(2), java.lang.Integer.valueOf(3)})).size();
    }

    public static int viaArraysEmpty() {
        // @method viaArraysEmpty()I
        // @declaration a static method of `W3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.util.ArrayList((java.util.Collection) java.util.Arrays.asList((java.lang.Object[]) new java.lang.Integer[0])).size();
    }

    public static java.lang.String viaClassLit() {
        // @method viaClassLit()Ljava/lang/String;
        // @declaration a static method of `W3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return nameOf(W3.class);
    }

    // jarde: generic Signature projection refused for `nameOf(Ljava/lang/Class;)Ljava/lang/String;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    static java.lang.String nameOf(java.lang.Class arg0) {
        // @method nameOf(Ljava/lang/Class;)Ljava/lang/String;
        // @declaration a static method of `W3`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.getSimpleName();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `W3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("" + viaArrays() + ":" + viaArraysEmpty() + ":" + viaClassLit());
        return;
    }
}
