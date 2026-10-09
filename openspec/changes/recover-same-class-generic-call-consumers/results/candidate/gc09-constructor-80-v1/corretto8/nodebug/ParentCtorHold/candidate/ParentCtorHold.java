// jarde: presentation of `ParentCtorHold` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/util/Vector<TT;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): parameterized or nested parent needs a separate inherited-member proof
public class ParentCtorHold extends java.util.Vector {
    // jarde: field Signature projection refused for `vLjava/lang/Object;`: unsupported (jvm_signature_scope_unproved): type variable `T` is not declared in the available Signature scope
    public java.lang.Object v;

    public ParentCtorHold(java.lang.Object arg1) {
        // @method <init>(Ljava/lang/Object;)V
        // @declaration a constructor of `ParentCtorHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super(1);
        this.v = arg1;
        return;
    }
}
