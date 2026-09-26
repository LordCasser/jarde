// jarde: presentation of `AnonymousInterfaceBasic` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class AnonymousInterfaceBasic extends java.lang.Object {
    public AnonymousInterfaceBasic() {
        // @method <init>()V
        // @declaration a constructor of `AnonymousInterfaceBasic`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static I make() {
        // @method make()LI;
        // @declaration a static method of `AnonymousInterfaceBasic`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new AnonymousInterfaceBasic$1();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `AnonymousInterfaceBasic`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(make().value());
        return;
    }
}
