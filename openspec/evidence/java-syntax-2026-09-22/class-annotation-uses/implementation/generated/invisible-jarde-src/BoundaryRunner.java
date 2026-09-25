// jarde: presentation of `BoundaryRunner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class BoundaryRunner extends java.lang.Object {
    public BoundaryRunner() {
        // @method <init>()V
        // @declaration a constructor of `BoundaryRunner`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `BoundaryRunner`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(HiddenTarget.class.getDeclaredAnnotations().length);
        java.lang.System.out.println((java.lang.Object) HiddenTarget.class.getAnnotation(HiddenTag.class));
        return;
    }
}
