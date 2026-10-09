// jarde: presentation of `BridgeBase` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
class BridgeBase<T> extends java.lang.Object {
    BridgeBase() {
        // @method <init>()V
        // @declaration a constructor of `BridgeBase`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public T apply(T x) {
        // jarde: generic Signature `(TT;)TT;` projected after descriptor erasure and same-run AST/SSA parameter-return proof
        // @method apply(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `BridgeBase`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return x;
    }
}
