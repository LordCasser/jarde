// jarde: presentation of `ListWrong` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class ListWrong<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `vLjava/util/List;`: unsupported (field_generic_write_source_unproved): writer put(Ljava/util/List;)V@2 is not source-assignable to the projected field type
    public java.util.List v;

    public ListWrong() {
        // @method <init>()V
        // @declaration a constructor of `ListWrong`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void put(java.util.List<java.lang.String> arg1) {
        // jarde: generic Signature `(Ljava/util/List<Ljava/lang/String;>;)V` projected after descriptor erasure and same-run AST/Code/SSA straight-line void body proof; same-class call binding proved
        // @method put(Ljava/util/List;)V
        // @declaration an instance method of `ListWrong`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.v = arg1;
        return;
    }
}
