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

    static java.lang.String run(int input) {
        // @method run(I)Ljava/lang/String;
        // @declaration a static method of `DequeThenBuilder`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.util.ArrayDeque queue = new java.util.ArrayDeque();
        queue.add((java.lang.Object) java.lang.Integer.valueOf(input));
        queue.add((java.lang.Object) java.lang.Integer.valueOf(input + 1));
        int size = queue.size();
        java.lang.StringBuilder out = new java.lang.StringBuilder();
        out.append(size).append(':').append(input);
        return out.toString();
    }

    public static void main(java.lang.String[] x) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `DequeThenBuilder`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) run(5));
        return;
    }
}
