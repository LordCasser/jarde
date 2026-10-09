// jarde: presentation of `MutatedReceiver` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class MutatedReceiver<T> extends java.lang.Object {
    public MutatedReceiver() {
        // @method <init>()V
        // @declaration a constructor of `MutatedReceiver`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `relay(Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public java.lang.Object relay(java.lang.Object x) {
        // @method relay(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `MutatedReceiver`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.util.ArrayList receiver = new java.util.ArrayList();
        java.util.ArrayList raw = new java.util.ArrayList();
        raw.add(x);
        receiver = raw;
        return receiver.get(0);
    }
}
