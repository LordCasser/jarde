// jarde: presentation of `MethodShadow` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class MethodShadow<T> extends java.lang.Object {
    public MethodShadow() {
        // @method <init>()V
        // @declaration a constructor of `MethodShadow`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public <T extends java.lang.Number> T relay(T arg1) {
        // jarde: generic Signature `<T:Ljava/lang/Number;>(TT;)TT;` projected after descriptor erasure and same-run complete AST/Code/SSA formal and return-consumer proof; same-class call binding proved
        // @method relay(Ljava/lang/Number;)Ljava/lang/Number;
        // @declaration an instance method of `MethodShadow`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.identity(arg1);
    }

    public <U extends java.lang.Number> U identity(U arg1) {
        // jarde: generic Signature `<U:Ljava/lang/Number;>(TU;)TU;` projected after descriptor erasure and same-run complete AST/Code/SSA formal and return-consumer proof; same-class call binding proved
        // @method identity(Ljava/lang/Number;)Ljava/lang/Number;
        // @declaration an instance method of `MethodShadow`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return arg1;
    }
}
