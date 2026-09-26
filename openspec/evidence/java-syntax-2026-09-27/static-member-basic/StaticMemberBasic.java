// jarde: presentation of `StaticMemberBasic` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class StaticMemberBasic extends java.lang.Object {
    public StaticMemberBasic() {
        // @method <init>()V
        // @declaration a constructor of `StaticMemberBasic`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static StaticMemberBasic$Leaf make() {
        // @method make()LStaticMemberBasic$Leaf;
        // @declaration a static method of `StaticMemberBasic`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new StaticMemberBasic$Leaf();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `StaticMemberBasic`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("" + make().value() + ":" + Named$Top.value());
        return;
    }
}
