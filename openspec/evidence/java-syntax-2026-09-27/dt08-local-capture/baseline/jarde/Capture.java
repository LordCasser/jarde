// jarde: presentation of `p/Capture` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

public class Capture extends java.lang.Object {
    public Capture() {
        // @method <init>()V
        // @declaration a constructor of `p.Capture`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.Runnable create(double arg0) {
        // @method create(D)Ljava/lang/Runnable;
        // @declaration a static method of `p.Capture`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new p.Capture$1(arg0);
    }
}
