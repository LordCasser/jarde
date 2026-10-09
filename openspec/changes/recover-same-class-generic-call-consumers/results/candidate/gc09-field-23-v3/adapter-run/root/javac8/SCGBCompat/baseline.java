// jarde: presentation of `SCGBCompat` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T::Ljava/lang/Comparable<TT;>;>Ljava/lang/Object;` projected after physical parent erasure proof
public class SCGBCompat<T extends java.lang.Comparable<T>> extends java.lang.Object {
    // jarde: field Signature `Ljava/util/Map<Ljava/lang/String;Ljava/util/List<TT;>;>;` projected after descriptor erasure and same-class uses at <init>()V@12, main([Ljava/lang/String;)V@9, main([Ljava/lang/String;)V@35
    private java.util.Map<java.lang.String, java.util.List<T>> index;

    public SCGBCompat() {
        // @method <init>()V
        // @declaration a constructor of `SCGBCompat`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.index = new java.util.HashMap();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `SCGBCompat`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        SCGBCompat local1;
        local1 = new SCGBCompat();
        if (local1.index != null) {
            java.lang.System.out.println("index:true");
        } else {
            java.lang.System.out.println("index:false");
        }
        java.util.Map local2 = local1.index;
        if ((java.lang.Object) local2 instanceof java.util.Map) {
            java.lang.System.out.println("read:true");
        } else {
            java.lang.System.out.println("read:false");
        }
        return;
    }
}
