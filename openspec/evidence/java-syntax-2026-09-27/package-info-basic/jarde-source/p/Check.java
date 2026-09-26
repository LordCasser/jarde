// jarde: presentation of `p/Check` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

public class Check extends java.lang.Object {
    public Check() {
        // @method <init>()V
        // @declaration a constructor of `p.Check`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `p.Check`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.Class.forName("p.package-info");
        java.lang.System.out.println(p.Check.class.getPackage().isAnnotationPresent(java.lang.Deprecated.class));
        return;
    }
}
