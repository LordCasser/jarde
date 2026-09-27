// jarde: presentation of `ListToIterable` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ListToIterable extends java.lang.Object {
    public ListToIterable() {
        // @method <init>()V
        // @declaration a constructor of `ListToIterable`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void consume(java.lang.Iterable values) {
        // @method consume(Ljava/lang/Iterable;)V
        // @declaration a static method of `ListToIterable`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("called");
        return;
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ListToIterable`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        consume((java.lang.Iterable) java.util.Arrays.asList((java.lang.Object[]) new java.lang.String[]{"a", "b", "c"}));
        return;
    }
}
