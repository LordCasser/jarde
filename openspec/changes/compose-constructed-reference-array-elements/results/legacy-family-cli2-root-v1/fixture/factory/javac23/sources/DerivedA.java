// jarde: presentation of `DerivedA` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class DerivedA extends Mid {
    public DerivedA(int arg1) {
        // @method <init>(I)V
        // @declaration a constructor of `DerivedA`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super(arg1);
        return;
    }
}
