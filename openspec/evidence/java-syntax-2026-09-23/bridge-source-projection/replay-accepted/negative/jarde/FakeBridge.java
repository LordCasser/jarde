// jarde: presentation of `FakeBridge` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class FakeBridge extends java.lang.Object {
    private static int calls;

    public FakeBridge() {
        // @method <init>()V
        // @declaration a constructor of `FakeBridge`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public java.lang.String get() {
        // @method get()Ljava/lang/String;
        // @declaration an instance method of `FakeBridge`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return "value";
    }

    public java.lang.Object get() {
        // @method get()Ljava/lang/Object;
        // @declaration an instance method of `FakeBridge`, member flags 0x1041
        // recovered from bytecode; presentation is not claimed to compile
        FakeBridge.calls = FakeBridge.calls + 1;
        return this.get();
    }

    public static int count() {
        // @method count()I
        // @declaration a static method of `FakeBridge`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return FakeBridge.calls;
    }
}
