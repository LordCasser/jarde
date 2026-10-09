// jarde: presentation of `RawBoundWriter` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class RawBoundWriter extends java.lang.Object {
    // jarde: field Signature `Ljava/util/Map<Ljava/lang/String;Ljava/lang/String;>;` projected after descriptor erasure and same-class uses at put(Ljava/util/HashMap;Z)V@2
    public java.util.Map<java.lang.String, java.lang.String> v;

    public RawBoundWriter() {
        // @method <init>()V
        // @declaration a constructor of `RawBoundWriter`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public <R extends java.util.HashMap> void put(R arg1, boolean arg2) {
        // jarde: generic Signature `<R:Ljava/util/HashMap;>(TR;Z)V` projected after descriptor erasure and same-run complete straight-line AST/Code/SSA parameter-use and Signature erasure proof
        // @method put(Ljava/util/HashMap;Z)V
        // @declaration an instance method of `RawBoundWriter`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.v = arg1;
        return;
    }
}
