// jarde: presentation of `RawReceiver` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class RawReceiver<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `valueLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer rawParameter(LRawReceiver;Ljava/lang/Object;)V@2 is not source-assignable to the projected field type
    public java.lang.Object value;

    public RawReceiver() {
        // @method <init>()V
        // @declaration a constructor of `RawReceiver`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void rawParameter(RawReceiver arg0, java.lang.Object arg1) {
        // @method rawParameter(LRawReceiver;Ljava/lang/Object;)V
        // @declaration a static method of `RawReceiver`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0.value = arg1;
        return;
    }

    public static void staticRawLocal(RawReceiver arg0, java.lang.Object arg1) {
        // @method staticRawLocal(LRawReceiver;Ljava/lang/Object;)V
        // @declaration a static method of `RawReceiver`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        RawReceiver local2 = arg0;
        local2.value = arg1;
        return;
    }

    public void instanceRawLocal(java.lang.Object arg1) {
        // @method instanceRawLocal(Ljava/lang/Object;)V
        // @declaration an instance method of `RawReceiver`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.value = arg1;
        return;
    }

    public void thisReceiver(T arg1) {
        // jarde: generic Signature `(TT;)V` projected after descriptor erasure and same-run AST/Code/SSA straight-line void body proof
        // @method thisReceiver(Ljava/lang/Object;)V
        // @declaration an instance method of `RawReceiver`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.value = arg1;
        return;
    }

    // jarde: generic Signature projection refused for `parameterizedReceiver(LRawReceiver;Ljava/lang/Object;)V`: unsupported (ordinary_generic_source_unproved): return source is not a proven parameter value or selected member creation
    public void parameterizedReceiver(RawReceiver arg1, java.lang.Object arg2) {
        // @method parameterizedReceiver(LRawReceiver;Ljava/lang/Object;)V
        // @declaration an instance method of `RawReceiver`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        arg1.value = arg2;
        return;
    }

    // jarde: generic Signature projection refused for `shadowMethodT(LRawReceiver;Ljava/lang/Object;)V`: unsupported (generic_source_shape_unproved): generic void body must be `<T extends B> void` with physical `(B, boolean)`, exact erasure, and two same-run parameter slots
    public static void shadowMethodT(RawReceiver arg0, java.lang.Object arg1) {
        // @method shadowMethodT(LRawReceiver;Ljava/lang/Object;)V
        // @declaration a static method of `RawReceiver`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0.value = arg1;
        return;
    }

    // jarde: generic Signature projection refused for `read()Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public java.lang.Object read() {
        // @method read()Ljava/lang/Object;
        // @declaration an instance method of `RawReceiver`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.value;
    }
}
