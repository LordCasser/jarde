// jarde: presentation of `TypedReceiver` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class TypedReceiver<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `valueLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer put(LTypedReceiver;Ljava/lang/Object;)V@2 is not source-assignable to the projected field type
    public java.lang.Object value;

    public TypedReceiver() {
        // @method <init>()V
        // @declaration a constructor of `TypedReceiver`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `put(LTypedReceiver;Ljava/lang/Object;)V`: unsupported (ordinary_generic_source_unproved): return source is not a proven parameter value or selected member creation
    public void put(TypedReceiver receiver, java.lang.Object value) {
        // @method put(LTypedReceiver;Ljava/lang/Object;)V
        // @declaration an instance method of `TypedReceiver`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        receiver.value = value;
        return;
    }

    // jarde: no body: the member `observe(LTypedReceiver;)V` is declared native and its declaration carries no Code attribute
    // jarde: generic Signature `(LTypedReceiver<TT;>;)V` projected after descriptor erasure and same-run AST/SSA parameter-return proof
    public native void observe(TypedReceiver<T> arg1);
}
