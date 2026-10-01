// jarde: presentation of `R1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class R1 extends java.lang.Object {
    public R1() {
        // @method <init>()V
        // @declaration a constructor of `R1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String ops(java.lang.String arg0) {
        // @method ops(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `R1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        local1 = new java.lang.StringBuilder();
        java.lang.String local2 = arg0 + "x";
        local1.append(local2 != arg0);
        return local1.toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `R1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) ops("abcd"));
        java.lang.System.out.println((java.lang.String) ops("banana"));
        return;
    }
}
