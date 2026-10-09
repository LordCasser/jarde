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

    static java.lang.String run(int arg0) {
        // @method run(I)Ljava/lang/String;
        // @declaration a static method of `MapThenBuilder`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.util.HashMap local2 = new java.util.HashMap();
        local2.put((java.lang.Object) "k", (java.lang.Object) java.lang.Integer.valueOf(arg0));
        int local1 = ((java.lang.Integer) local2.get((java.lang.Object) "k")).intValue();
        java.lang.StringBuilder local2_2 = new java.lang.StringBuilder();
        local2_2.append(local1).append(':').append(arg0);
        return local2_2.toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `MapThenBuilder`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) run(6));
        return;
    }
}
