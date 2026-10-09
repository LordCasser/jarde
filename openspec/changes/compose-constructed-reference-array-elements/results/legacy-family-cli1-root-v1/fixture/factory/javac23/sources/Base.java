// jarde: presentation of `Base` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Base extends java.lang.Object implements LocalInterface {
    private final int value;

    public Base(int arg1) {
        // @method <init>(I)V
        // @declaration a constructor of `Base`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.value = arg1;
        return;
    }

    public int value() {
        // @method value()I
        // @declaration an instance method of `Base`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.value;
    }
}
