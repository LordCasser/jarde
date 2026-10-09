// jarde: presentation of `VarargsCall` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class VarargsCall<T> extends java.lang.Object {
    public VarargsCall() {
        // @method <init>()V
        // @declaration a constructor of `VarargsCall`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `first([Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    @java.lang.SafeVarargs
    public final java.lang.Object first(java.lang.Object... xs) {
        // @method first([Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `VarargsCall`, member flags 0x0091
        // recovered from bytecode; presentation is not claimed to compile
        return xs[0];
    }

    // jarde: generic Signature projection refused for `relay(Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public java.lang.Object relay(java.lang.Object x) {
        // @method relay(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `VarargsCall`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.first(x);
    }
}
