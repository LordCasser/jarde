// jarde: presentation of `p/Subject` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

public class Subject extends java.lang.Object {
    public static int value;

    public Subject() {
        // @method <init>()V
        // @declaration a constructor of `p.Subject`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static p.Base make() {
        return new p.Base() {
            {
                p.Subject.value = 1;
            }
            public void run() {
                p.Subject.value = p.Subject.value + 7;
                return;
            }
        };
    }
}
