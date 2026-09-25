// jarde: presentation of `GenericMethodProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class GenericMethodProbe extends java.lang.Object {
    public GenericMethodProbe() {
        // @method <init>()V
        // @declaration a constructor of `GenericMethodProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.Number choose(java.lang.Number arg0, java.lang.Number arg1, boolean arg2) {
        // @method choose(Ljava/lang/Number;Ljava/lang/Number;Z)Ljava/lang/Number;
        // @declaration a static method of `GenericMethodProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg2 ? arg0 : arg1;
    }
}
