// jarde: presentation of `DerivedB` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class DerivedB extends Base {
    public DerivedB() {
        // @method <init>()V
        // @declaration a constructor of `DerivedB`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    @Override
    public java.lang.String name() {
        // @method name()Ljava/lang/String;
        // @declaration an instance method of `DerivedB`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return "b";
    }
}
