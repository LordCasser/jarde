// jarde: presentation of `em02/PrivateBase` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em02;

public class PrivateBase extends java.lang.Object {
    public PrivateBase() {
        // @method <init>()V
        // @declaration a constructor of `em02.PrivateBase`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private int ping() {
        // @method ping()I
        // @declaration an instance method of `em02.PrivateBase`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return 1;
    }

    public int basePing() {
        // @method basePing()I
        // @declaration an instance method of `em02.PrivateBase`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.ping();
    }
}
