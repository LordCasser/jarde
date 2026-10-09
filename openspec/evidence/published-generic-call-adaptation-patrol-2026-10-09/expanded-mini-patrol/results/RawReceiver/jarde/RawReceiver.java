// jarde: presentation of `RawReceiver` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class RawReceiver<T> extends java.lang.Object {
    public RawReceiver$Box box;

    public RawReceiver() {
        // @method <init>()V
        // @declaration a constructor of `RawReceiver`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.box = new RawReceiver$Box();
        return;
    }

    // jarde: generic Signature projection refused for `relay(Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public java.lang.Object relay(java.lang.Object x) {
        // @method relay(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `RawReceiver`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.box.id(x);
    }

    public static void main(java.lang.String[] a) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `RawReceiver`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.Object m = new java.lang.Object();
        RawReceiver c = new RawReceiver();
        java.lang.System.out.println("behavior.marker=" + (c.relay(m) == m));
        return;
    }
}
