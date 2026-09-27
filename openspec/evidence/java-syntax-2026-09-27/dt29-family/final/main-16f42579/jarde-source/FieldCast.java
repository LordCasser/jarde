// jarde: presentation of `dt29/FieldCast` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package dt29;

public class FieldCast extends java.lang.Object {
    public FieldCast() {
        // @method <init>()V
        // @declaration a constructor of `dt29.FieldCast`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String run() {
        // @method run()Ljava/lang/String;
        // @declaration a static method of `dt29.FieldCast`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        dt29.FieldCast$B local0 = new dt29.FieldCast$B();
        local0.self(true);
        java.lang.String local1 = bits((dt29.FieldCast$A) local0);
        new dt29.FieldCast$C().set(local0, false);
        java.lang.String local2 = bits((dt29.FieldCast$A) local0);
        new dt29.FieldCast$D((dt29.FieldCast$1) null).set(local0, true);
        return local1 + ":" + local2 + ":" + bits((dt29.FieldCast$A) local0);
    }

    private static java.lang.String bits(dt29.FieldCast$A arg0) {
        // @method bits(Ldt29/FieldCast$A;)Ljava/lang/String;
        // @declaration a static method of `dt29.FieldCast`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return (arg0.publicField ? "1" : "0") + (arg0.protectedField ? "1" : "0") + (arg0.packagePrivateField ? "1" : "0") + (dt29.FieldCast$A.access$000(arg0) ? "1" : "0");
    }
}
