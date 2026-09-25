// jarde: presentation of `minimal/UseSimple` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package minimal;

public final class UseSimple extends java.lang.Object {
    public UseSimple() {
        // @method <init>()V
        // @declaration a constructor of `minimal.UseSimple`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.Object make(minimal.Outer arg0, int arg1) {
        // @method make(Lminimal/Outer;I)Ljava/lang/Object;
        // @declaration a static method of `minimal.UseSimple`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.new Inner<>((java.lang.Object) java.lang.Integer.valueOf(arg1));
    }
}
