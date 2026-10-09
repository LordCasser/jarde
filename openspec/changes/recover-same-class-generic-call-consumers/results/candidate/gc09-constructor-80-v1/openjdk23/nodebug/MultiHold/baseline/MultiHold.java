// jarde: presentation of `MultiHold` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;U::Ljava/lang/CharSequence;>Ljava/lang/Object;` projected after physical parent erasure proof
public class MultiHold<T, U extends java.lang.CharSequence> extends java.lang.Object {
    // jarde: field Signature `TT;` projected after descriptor erasure and same-class uses at <init>(Ljava/lang/Object;Ljava/lang/CharSequence;)V@6
    public T first;

    // jarde: field Signature `TU;` projected after descriptor erasure and same-class uses at <init>(Ljava/lang/Object;Ljava/lang/CharSequence;)V@11
    public U second;

    public MultiHold(T arg1, U arg2) {
        // jarde: generic Signature `(TT;TU;)V` projected after descriptor erasure and same-run AST/Code/SSA direct field-initializer proof
        // @method <init>(Ljava/lang/Object;Ljava/lang/CharSequence;)V
        // @declaration a constructor of `MultiHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.first = arg1;
        this.second = arg2;
        return;
    }
}
