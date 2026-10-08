// jarde: presentation of `ParameterShift` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class ParameterShift<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `vLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer put(JDLjava/lang/Object;)V@3 is not source-assignable to the projected field type
    public java.lang.Object v;

    public ParameterShift() {
        // @method <init>()V
        // @declaration a constructor of `ParameterShift`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `put(JDLjava/lang/Object;)V`: unsupported (ordinary_generic_source_unproved): return source is not a proven parameter value or selected member creation
    public void put(long arg1, double arg3, java.lang.Object arg5) {
        // @method put(JDLjava/lang/Object;)V
        // @declaration an instance method of `ParameterShift`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.v = arg5;
        return;
    }
}
