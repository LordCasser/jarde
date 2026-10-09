// jarde: presentation of `DirectB` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class DirectB extends Base {
    public DirectB(java.lang.String arg1) {
        // @method <init>(Ljava/lang/String;)V
        // @declaration a constructor of `DirectB`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1);
        Main.event("ctor:DirectB", arg1);
        return;
    }
}
