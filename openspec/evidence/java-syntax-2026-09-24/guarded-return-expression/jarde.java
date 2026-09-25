// jarde: presentation of `Locked` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Locked extends java.lang.Object {
    int n;

    public Locked() {
        // @method <init>()V
        // @declaration a constructor of `Locked`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    int locked() {
        // @method locked()I
        // @declaration an instance method of `Locked`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        synchronized (this) {
            return this.n;
        }
    }
}
