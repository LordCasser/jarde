// jarde: presentation of `WideStoredHold` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class WideStoredHold<T> extends java.lang.Object {
    public long sequence;

    public double weight;

    // jarde: field Signature projection refused for `vLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer <init>(JDLjava/lang/Object;)V@17 is not source-assignable to the projected field type
    public java.lang.Object v;

    // jarde: generic Signature projection refused for `<init>(JDLjava/lang/Object;)V`: unsupported (ordinary_generic_source_unproved): member flags, annotation positions, or source name cannot be preserved
    public WideStoredHold(long arg1, double arg3, java.lang.Object arg5) {
        // @method <init>(JDLjava/lang/Object;)V
        // @declaration a constructor of `WideStoredHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.sequence = arg1;
        this.weight = arg3;
        this.v = arg5;
        return;
    }
}
