// jarde: presentation of `N1$Numbers$NumString` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `Ljava/lang/Enum<LN1$Numbers$NumString;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum N1$Numbers$NumString {
    ONE("one"),
    TWO("two");

    private final java.lang.String name;

    private N1$Numbers$NumString(java.lang.String arg0) {
        this.name = arg0;
    }

    public java.lang.String getName() {
        // @method getName()Ljava/lang/String;
        // @declaration an instance method of `N1$Numbers$NumString`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.name;
    }
}
