// jarde: presentation of `nested/UseInner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package nested;

public final class UseInner extends java.lang.Object {
    public UseInner() {
        // @method <init>()V
        // @declaration a constructor of `nested.UseInner`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.Object make(nested.SimpleOuter arg0, int arg1) {
        // @method make(Lnested/SimpleOuter;I)Ljava/lang/Object;
        // @declaration a static method of `nested.UseInner`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.new Inner(nested.SimpleOuter.mark("A", arg1));
    }
}
