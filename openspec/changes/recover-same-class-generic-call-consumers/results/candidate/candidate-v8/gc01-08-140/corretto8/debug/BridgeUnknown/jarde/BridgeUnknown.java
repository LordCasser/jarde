// jarde: presentation of `BridgeUnknown` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `LBridgeBase<Ljava/lang/String;>;` projected after physical parent erasure proof
public class BridgeUnknown extends BridgeBase<java.lang.String> {
    public BridgeUnknown() {
        // @method <init>()V
        // @declaration a constructor of `BridgeUnknown`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public java.lang.String apply(java.lang.String x) {
        // @method apply(Ljava/lang/String;)Ljava/lang/String;
        // @declaration an instance method of `BridgeUnknown`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return x;
    }

    public java.lang.Object relay(java.lang.Object x) {
        // @method relay(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `BridgeUnknown`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.apply((java.lang.String) x);
    }

    // jarde: projected bridge `apply(Ljava/lang/Object;)Ljava/lang/Object;` at physical method record 3: bridge@1 proved a pure call at BCI 5 to source declaration `apply(Ljava/lang/String;)Ljava/lang/String;` at method record 1; the resolved erased contract lets javac regenerate the bridge
}
