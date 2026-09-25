// jarde: presentation of `MemberRunner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class MemberRunner extends java.lang.Object {
    public MemberRunner() {
        // @method <init>()V
        // @declaration a constructor of `MemberRunner`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `MemberRunner`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        Object local1 = MemberTagged.class.getDeclaredFields()[0];
        Object local2 = MemberTagged.class.getDeclaredMethods()[0];
        java.lang.System.out.println(((java.lang.reflect.Field) local1).isAnnotationPresent(java.lang.Deprecated.class));
        java.lang.System.out.println(((java.lang.reflect.Method) local2).isAnnotationPresent(java.lang.Deprecated.class));
        java.lang.System.out.println(((java.lang.reflect.Method) local2).getParameterAnnotations()[0].length);
        java.lang.System.out.println(new MemberTagged().value(2));
        return;
    }
}
