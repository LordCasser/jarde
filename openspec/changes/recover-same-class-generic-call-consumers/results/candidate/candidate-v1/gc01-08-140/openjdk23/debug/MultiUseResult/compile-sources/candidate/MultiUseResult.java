// jarde: presentation of `MultiUseResult` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class MultiUseResult<T> extends java.lang.Object {
    public java.lang.Object observed;

    public MultiUseResult() {
        // @method <init>()V
        // @declaration a constructor of `MultiUseResult`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `identity(Ljava/lang/Object;)Ljava/lang/Object;`: same-class generic call dependency did not close over every incoming use
    public java.lang.Object identity(java.lang.Object x) {
        // @method identity(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `MultiUseResult`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return x;
    }

    public void observe(java.lang.Object x) {
        // @method observe(Ljava/lang/Object;)V
        // @declaration an instance method of `MultiUseResult`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.observed = x;
        return;
    }

    // jarde: generic Signature projection refused for `relay(Ljava/lang/Object;)Ljava/lang/Object;`: same-class generic call dependency did not close over every incoming use
    public java.lang.Object relay(java.lang.Object x) {
        // @method relay(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `MultiUseResult`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.Object result = this.identity(x);
        this.observe(result);
        return result;
    }
}
