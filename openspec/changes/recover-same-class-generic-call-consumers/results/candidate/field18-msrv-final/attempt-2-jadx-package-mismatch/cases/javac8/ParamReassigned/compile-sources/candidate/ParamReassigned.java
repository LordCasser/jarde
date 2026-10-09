// jarde: presentation of `ParamReassigned` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class ParamReassigned<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `vLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer put(Ljava/lang/Object;Ljava/lang/Object;)V@4 has no closed SSA source
    public java.lang.Object v;

    public ParamReassigned() {
        // @method <init>()V
        // @declaration a constructor of `ParamReassigned`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `put(Ljava/lang/Object;Ljava/lang/Object;)V`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public void put(java.lang.Object arg1, java.lang.Object arg2) {
        // @method put(Ljava/lang/Object;Ljava/lang/Object;)V
        // @declaration an instance method of `ParamReassigned`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        arg1 = arg2;
        this.v = arg1;
        return;
    }
}
