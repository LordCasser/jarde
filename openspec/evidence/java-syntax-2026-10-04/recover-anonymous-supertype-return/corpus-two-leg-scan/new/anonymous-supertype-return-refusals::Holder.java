// jarde: presentation of `Holder` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class Holder extends java.lang.Object {
    Holder() {
        // @method <init>()V
        // @declaration a constructor of `Holder`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static interface Marker {
        // jarde: no body: the member `tag()Ljava/lang/String;` is declared abstract and its declaration carries no Code attribute
        public abstract java.lang.String tag();
    }
}
