// jarde: presentation of `VoidDirect` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class VoidDirect<T> extends java.lang.Object {
    public java.lang.Object seen;

    public int calls;

    public VoidDirect() {
        // @method <init>()V
        // @declaration a constructor of `VoidDirect`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `sink(Ljava/lang/Object;)V`: same-class generic call dependency did not close over every incoming use
    public void sink(java.lang.Object x) {
        // @method sink(Ljava/lang/Object;)V
        // @declaration an instance method of `VoidDirect`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.seen = x;
        this.calls++;
        return;
    }

    // jarde: generic Signature projection refused for `relay(Ljava/lang/Object;)V`: same-class generic call dependency did not close over every incoming use
    public void relay(java.lang.Object x) {
        // @method relay(Ljava/lang/Object;)V
        // @declaration an instance method of `VoidDirect`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.sink(x);
        return;
    }
}
