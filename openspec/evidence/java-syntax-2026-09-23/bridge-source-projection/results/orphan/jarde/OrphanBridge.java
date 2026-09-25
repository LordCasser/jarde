// jarde: presentation of `OrphanBridge` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class OrphanBridge extends java.lang.Object {
    public OrphanBridge() {
        // @method <init>()V
        // @declaration a constructor of `OrphanBridge`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public java.lang.String get() {
        // @method get()Ljava/lang/String;
        // @declaration an instance method of `OrphanBridge`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return "value";
    }

    public java.lang.Object get() {
        // @method get()Ljava/lang/Object;
        // @declaration an instance method of `OrphanBridge`, member flags 0x1041
        // recovered from bytecode; presentation is not claimed to compile
        return this.get();
    }
}
