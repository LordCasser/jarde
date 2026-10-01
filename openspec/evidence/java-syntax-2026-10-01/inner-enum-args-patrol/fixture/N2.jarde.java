// jarde: presentation of `N2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class N2 extends java.lang.Object {
    public N2() {
        // @method <init>()V
        // @declaration a constructor of `N2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `N2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(N2$Operation.PLUS.apply(3, 4));
        java.lang.System.out.println(N2$Operation.MINUS.apply(9, 2));
        return;
    }
}
