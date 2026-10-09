// jarde: presentation of `GenericNullObjectContextProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class GenericNullObjectContextProbe extends java.lang.Object {
    public GenericNullObjectContextProbe() {
        // @method <init>()V
        // @declaration a constructor of `GenericNullObjectContextProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public <T extends java.lang.Number> T value() {
        // jarde: generic Signature `<T:Ljava/lang/Number;>()TT;` projected after descriptor erasure and same-run AST/Code/SSA exact null-return and method-local Signature scope/erasure proof; same-class call binding proved
        // @method value()Ljava/lang/Number;
        // @declaration an instance method of `GenericNullObjectContextProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return null;
    }

    public java.lang.Number value(java.lang.Number arg1) {
        // @method value(Ljava/lang/Number;)Ljava/lang/Number;
        // @declaration an instance method of `GenericNullObjectContextProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1;
    }

    public java.lang.Object caller() {
        // @method caller()Ljava/lang/Object;
        // @declaration an instance method of `GenericNullObjectContextProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.value();
    }
}
