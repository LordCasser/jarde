// jarde: presentation of `Base` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
abstract class Base extends java.lang.Object {
    Base() {
        // @method <init>()V
        // @declaration a constructor of `Base`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        AnonymousSuperDispatch.inBaseConstructor = true;
        this.observe();
        // @bytecode 12 15 18 20 23 26 29 30 33 34 37 38 41
        // the short-circuit branches at BCI 15 and 26 reach a shared value consumer at BCI 34, but this slice has no SSA proof for that value; the complete region is quoted
    }

    // jarde: no body: the member `observe()V` is declared abstract and its declaration carries no Code attribute
    abstract void observe();
}
