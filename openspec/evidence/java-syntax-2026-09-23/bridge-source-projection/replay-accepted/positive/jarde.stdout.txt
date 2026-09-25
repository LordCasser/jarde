// jarde: presentation of `BridgeProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class BridgeProbe extends java.lang.Object implements BridgeApi {
    public BridgeProbe() {
        // @method <init>()V
        // @declaration a constructor of `BridgeProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public java.lang.String get() {
        // @method get()Ljava/lang/String;
        // @declaration an instance method of `BridgeProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return "value";
    }

    public static java.lang.String observe() {
        // @method observe()Ljava/lang/String;
        // @declaration a static method of `BridgeProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        BridgeProbe api = new BridgeProbe();
        return (java.lang.String) api.get();
    }

    // jarde: projected bridge `get()Ljava/lang/Object;` at physical method record 3: bridge@1 proved a pure call at BCI 1 to source declaration `get()Ljava/lang/String;` at method record 1; the resolved erased contract lets javac regenerate the bridge
}
