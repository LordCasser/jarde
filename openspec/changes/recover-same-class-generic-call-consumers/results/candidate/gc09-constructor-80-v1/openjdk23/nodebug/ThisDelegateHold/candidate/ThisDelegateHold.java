// jarde: presentation of `ThisDelegateHold` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class ThisDelegateHold<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `vLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer <init>(Ljava/lang/Object;I)V@6 is not source-assignable to the projected field type
    public java.lang.Object v;

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/Object;)V`: same-class generic call dependency did not close over every incoming use
    public ThisDelegateHold(java.lang.Object arg1) {
        // @method <init>(Ljava/lang/Object;)V
        // @declaration a constructor of `ThisDelegateHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this(arg1, 0);
        return;
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/Object;I)V`: same-class generic call dependency did not close over every incoming use
    public ThisDelegateHold(java.lang.Object arg1, int arg2) {
        // @method <init>(Ljava/lang/Object;I)V
        // @declaration a constructor of `ThisDelegateHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.v = arg1;
        return;
    }
}
