// jarde: presentation of `FieldSetter` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class FieldSetter<T> extends java.lang.Object {
    // jarde: field Signature `TT;` projected after descriptor erasure and same-class uses at set(Ljava/lang/Object;)V@2
    public T value;

    public FieldSetter() {
        // @method <init>()V
        // @declaration a constructor of `FieldSetter`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void set(T x) {
        // jarde: generic Signature `(TT;)V` projected after descriptor erasure and same-run AST/Code/SSA straight-line void body proof
        // @method set(Ljava/lang/Object;)V
        // @declaration an instance method of `FieldSetter`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.value = x;
        return;
    }
}
