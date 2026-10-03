// jarde: presentation of `WCallI1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class WCallI1 extends java.lang.Object {
    public WCallI1() {
        // @method <init>()V
        // @declaration a constructor of `WCallI1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `WCallI1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(M.sv());
        return;
    }

    static interface M {
        public static int sv() {
            // @method sv()I
            // @declaration an interface's static method of `WCallI1$M`, member flags 0x0009
            // recovered from bytecode; presentation is not claimed to compile
            return 8;
        }

        // jarde: no body: the member `v()I` is declared abstract and its declaration carries no Code attribute
        public abstract int v();
    }
}
