// jarde: presentation of `Main` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Main extends java.lang.Object {
    public Main() {
        // @method <init>()V
        // @declaration a constructor of `Main`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Main`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.CharSequence[] local1 = new java.lang.CharSequence[]{"x", new java.lang.StringBuilder("y")};
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(local1.length).append(":").append((java.lang.Object) local1[0]).append(":").append((java.lang.Object) local1[1]).toString());
        return;
    }
}
