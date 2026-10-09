// jarde: presentation of `GenericAbstractReceiverProbe` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public abstract class GenericAbstractReceiverProbe extends java.lang.Object {
    public GenericAbstractReceiverProbe() {
        // @method <init>()V
        // @declaration a constructor of `GenericAbstractReceiverProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: no body: the member `target(Ljava/lang/Number;)Ljava/lang/Number;` is declared abstract and its declaration carries no Code attribute
    // jarde: generic Signature `<T:Ljava/lang/Number;>(TT;)TT;` projected after descriptor erasure and physical no-body declaration proof; same-class call binding proved
    public abstract <T extends java.lang.Number> T target(T arg1);

    public static java.lang.Number call(GenericAbstractReceiverProbe arg0) {
        // @method call(LGenericAbstractReceiverProbe;)Ljava/lang/Number;
        // @declaration a static method of `GenericAbstractReceiverProbe`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.target((java.lang.Number) java.lang.Integer.valueOf(7));
    }

    public java.lang.Number own() {
        // @method own()Ljava/lang/Number;
        // @declaration an instance method of `GenericAbstractReceiverProbe`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.target((java.lang.Number) java.lang.Integer.valueOf(8));
    }
}
