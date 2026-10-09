// jarde: presentation of `CycleRelay` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class CycleRelay<T> extends java.lang.Object {
    public CycleRelay() {
        // @method <init>()V
        // @declaration a constructor of `CycleRelay`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `left(Ljava/lang/Object;Z)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public java.lang.Object left(java.lang.Object arg1, boolean arg2) {
        // @method left(Ljava/lang/Object;Z)Ljava/lang/Object;
        // @declaration an instance method of `CycleRelay`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        if (arg2) {
            return this.right(arg1, false);
        } else {
            return arg1;
        }
    }

    // jarde: generic Signature projection refused for `right(Ljava/lang/Object;Z)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public java.lang.Object right(java.lang.Object arg1, boolean arg2) {
        // @method right(Ljava/lang/Object;Z)Ljava/lang/Object;
        // @declaration an instance method of `CycleRelay`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        if (arg2) {
            return this.left(arg1, false);
        } else {
            return arg1;
        }
    }
}
