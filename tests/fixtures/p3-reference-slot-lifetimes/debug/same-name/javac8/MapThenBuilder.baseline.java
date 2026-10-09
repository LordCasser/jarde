// jarde: presentation of `MapThenBuilder` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class MapThenBuilder extends java.lang.Object {
    public MapThenBuilder() {
        // @method <init>()V
        // @declaration a constructor of `MapThenBuilder`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String run(int x) {
        // @method run(I)Ljava/lang/String;
        // @declaration a static method of `MapThenBuilder`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.util.HashMap v = new java.util.HashMap();
        v.put((java.lang.Object) "k", (java.lang.Object) java.lang.Integer.valueOf(x));
        int n = ((java.lang.Integer) v.get((java.lang.Object) "k")).intValue();
        v = new java.lang.StringBuilder();
        v.append(n).append(':').append(x);
        return v.toString();
    }

    public static void main(java.lang.String[] a) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `MapThenBuilder`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) run(6));
        return;
    }
}
