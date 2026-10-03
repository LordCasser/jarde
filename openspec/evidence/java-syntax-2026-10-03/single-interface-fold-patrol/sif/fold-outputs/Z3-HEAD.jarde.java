// jarde: presentation of `Z3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Z3 extends java.lang.Object {
    public Z3() {
        // @method <init>()V
        // @declaration a constructor of `Z3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `Z3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("ok");
        return;
    }

    static interface StrFn {
        // jarde: no body: the member `apply(Ljava/lang/String;)Ljava/lang/String;` is declared abstract and its declaration carries no Code attribute
        public abstract java.lang.String apply(java.lang.String arg1);
    }
}
