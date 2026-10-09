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

    static java.lang.String run(java.lang.String arg0) {
        // @method run(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `StringThenQueue`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.String local2 = arg0.trim();
        int local1 = local2.length();
        java.util.ArrayDeque local2_2 = new java.util.ArrayDeque();
        local2_2.add((java.lang.Object) arg0);
        local2_2.add((java.lang.Object) "!");
        return new java.lang.StringBuilder().append(local1).append(":").append((java.lang.String) local2_2.removeFirst()).append((java.lang.String) local2_2.removeLast()).toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `StringThenQueue`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) run(" q "));
        return;
    }
}
