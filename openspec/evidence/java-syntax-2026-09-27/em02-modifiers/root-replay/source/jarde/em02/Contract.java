// jarde: presentation of `em02/Contract` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em02;

public interface Contract {
    // jarde: no body: the member `value()I` is declared abstract and its declaration carries no Code attribute
    public abstract int value();

    public default int plusOne() {
        // @method plusOne()I
        // @declaration an interface's default method of `em02.Contract`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.value() + 1;
    }

    public static int five() {
        // @method five()I
        // @declaration an interface's static method of `em02.Contract`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return 5;
    }

    // jarde: no body: the member `alsoValue()I` is declared abstract and its declaration carries no Code attribute
    public abstract int alsoValue();
}
