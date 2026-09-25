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
        AnonymousSuperDispatch.capturedVisibleBeforeBaseReturns = (AnonymousSuperDispatch.inBaseConstructor ? "captured-value".equals((java.lang.Object) AnonymousSuperDispatch.observed) ? 1 : 0 : 0) % 2 != 0;
        AnonymousSuperDispatch.inBaseConstructor = false;
        return;
    }

    // jarde: no body: the member `observe()V` is declared abstract and its declaration carries no Code attribute
    abstract void observe();
}
