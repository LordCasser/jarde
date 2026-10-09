// jarde: presentation of `MethodHandleUse` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class MethodHandleUse<T> extends java.lang.Object {
    public MethodHandleUse() {
        // @method <init>()V
        // @declaration a constructor of `MethodHandleUse`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `identity(Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    public java.lang.Object identity(java.lang.Object x) {
        // @method identity(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `MethodHandleUse`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return x;
    }

    // jarde: generic Signature projection refused for `relay(Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public java.lang.Object relay(java.lang.Object x) {
        // @method relay(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `MethodHandleUse`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.util.function.Function f = this::identity;
        return f.apply(x);
    }
}
