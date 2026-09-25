// jarde: presentation of `MemberTagged` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class MemberTagged extends java.lang.Object {
    public int field;

    public MemberTagged() {
        // @method <init>()V
        // @declaration a constructor of `MemberTagged`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.field = 3;
        return;
    }

    public int value(int arg1) {
        // @method value(I)I
        // @declaration an instance method of `MemberTagged`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1 + this.field;
    }
}
