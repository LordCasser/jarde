// jarde: presentation of `SameErasureBinder` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Number;:Ljava/lang/Runnable;>Ljava/lang/Object;` projected after physical parent erasure proof
public class SameErasureBinder<T extends java.lang.Number & java.lang.Runnable> extends java.lang.Object {
    public SameErasureBinder() {
        // @method <init>()V
        // @declaration a constructor of `SameErasureBinder`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `sink(Ljava/lang/Number;)Ljava/lang/Number;`: unsupported (generic_source_shape_unproved): method flags, annotations, or existing source refusals cannot be preserved in this generic shape
    public java.lang.Number sink(java.lang.Number x) {
        // @method sink(Ljava/lang/Number;)Ljava/lang/Number;
        // @declaration an instance method of `SameErasureBinder`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return x;
    }

    // jarde: generic Signature projection refused for `relay(Ljava/lang/Number;)Ljava/lang/Number;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public java.lang.Number relay(java.lang.Number x) {
        // @method relay(Ljava/lang/Number;)Ljava/lang/Number;
        // @declaration an instance method of `SameErasureBinder`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.sink(x);
    }
}
