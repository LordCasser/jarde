// jarde: presentation of `StringThenQueue` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class StringThenQueue extends java.lang.Object {
    public StringThenQueue() {
        // @method <init>()V
        // @declaration a constructor of `StringThenQueue`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String run(java.lang.String x) {
        // @method run(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `StringThenQueue`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String v = x.trim();
        int n = v.length();
        v = new java.util.ArrayDeque();
        v.add((java.lang.Object) x);
        v.add((java.lang.Object) "!");
        return new java.lang.StringBuilder().append(n).append(":").append((java.lang.String) v.removeFirst()).append((java.lang.String) v.removeLast()).toString();
    }

    public static void main(java.lang.String[] a) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `StringThenQueue`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) run(" q "));
        return;
    }
}
