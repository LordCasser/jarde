// jarde: presentation of `ShadowMethodT` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class ShadowMethodT<T> extends java.lang.Object {
    // jarde: field Signature `TT;` projected after descriptor erasure and same-class uses at put(LShadowMethodT;Ljava/lang/Object;)V@2
    public T value;

    public ShadowMethodT() {
        // @method <init>()V
        // @declaration a constructor of `ShadowMethodT`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `put(LShadowMethodT;Ljava/lang/Object;)V`: same-class generic call dependency did not close over every incoming use
    public static void put(ShadowMethodT receiver, java.lang.Object value) {
        // @method put(LShadowMethodT;Ljava/lang/Object;)V
        // @declaration a static method of `ShadowMethodT`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        receiver.value = value;
        return;
    }

    // jarde: generic Signature projection refused for `observe(LShadowMethodT;Ljava/lang/Object;)V`: unsupported (generic_inherited_contract_unproved): no-body generic projection requires a top-level class or root interface with Object as its physical superclass and no interfaces
    // jarde: no body: the member `observe(LShadowMethodT;Ljava/lang/Object;)V` is declared native and its declaration carries no Code attribute
    public static native void observe(ShadowMethodT arg0, java.lang.Object arg1);
}
