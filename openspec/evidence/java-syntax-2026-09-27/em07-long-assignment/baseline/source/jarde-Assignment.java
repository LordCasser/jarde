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
        // jarde: not recovered: the recovery run for `setValue(J)J` produced no statement (explanation only); the artifact's own comment lines are below
        // @method setValue(J)J
        // @declaration an instance method of `em07.Assignment`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the instruction at BCI 2 is not part of the provable subset
        // @bytecode 3
        // the value at BCI 3 comes from an Other at BCI 2, which produces no expression this subset writes
        // @bytecode 6
        // the value at BCI 6 comes from an Other at BCI 2, which produces no expression this subset writes
    }

    public long getValue() {
        // @method getValue()J
        // @declaration an instance method of `em07.Assignment`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.value;
    }
}
