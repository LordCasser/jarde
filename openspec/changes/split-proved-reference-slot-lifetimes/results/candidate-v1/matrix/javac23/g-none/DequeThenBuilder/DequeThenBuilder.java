// jarde: presentation of `DequeThenBuilder` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class DequeThenBuilder extends java.lang.Object {
    public DequeThenBuilder() {
        // @method <init>()V
        // @declaration a constructor of `DequeThenBuilder`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String run(int arg0) {
        // @method run(I)Ljava/lang/String;
        // @declaration a static method of `DequeThenBuilder`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.util.ArrayDeque local2 = new java.util.ArrayDeque();
        local2.add((java.lang.Object) java.lang.Integer.valueOf(arg0));
        local2.add((java.lang.Object) java.lang.Integer.valueOf(arg0 + 1));
        int local1 = local2.size();
        java.lang.StringBuilder local2_2 = new java.lang.StringBuilder();
        local2_2.append(local1).append(':').append(arg0);
        return local2_2.toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `DequeThenBuilder`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) run(5));
        return;
    }
}
