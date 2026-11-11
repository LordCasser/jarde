// jarde: presentation of `em07/Assignment` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em07;

public class Assignment extends java.lang.Object {
    private long value;

    public Assignment() {
        // @method <init>()V
        // @declaration a constructor of `em07.Assignment`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public long setValue(long arg1) {
        // @method setValue(J)J
        // @declaration an instance method of `em07.Assignment`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.value = arg1;
        return arg1;
    }

    public long getValue() {
        // @method getValue()J
        // @declaration an instance method of `em07.Assignment`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.value;
    }
}
