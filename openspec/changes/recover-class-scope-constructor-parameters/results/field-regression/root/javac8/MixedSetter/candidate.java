// jarde: presentation of `MixedSetter` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class MixedSetter<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `vLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer putObject(Ljava/lang/Object;)V@2 is not source-assignable to the projected field type
    public java.lang.Object v;

    public MixedSetter() {
        // @method <init>()V
        // @declaration a constructor of `MixedSetter`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void putT(T arg1) {
        // jarde: generic Signature `(TT;)V` projected after descriptor erasure and same-run AST/Code/SSA straight-line void body proof
        // @method putT(Ljava/lang/Object;)V
        // @declaration an instance method of `MixedSetter`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.v = arg1;
        return;
    }

    public void putObject(java.lang.Object arg1) {
        // @method putObject(Ljava/lang/Object;)V
        // @declaration an instance method of `MixedSetter`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.v = arg1;
        return;
    }
}
