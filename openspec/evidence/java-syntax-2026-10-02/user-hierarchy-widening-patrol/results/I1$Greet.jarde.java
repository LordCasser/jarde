// jarde: presentation of `I1$Greet` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
interface I1$Greet {
    public default java.lang.String hello(java.lang.String arg1) {
        // @method hello(Ljava/lang/String;)Ljava/lang/String;
        // @declaration an interface's default method of `I1$Greet`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return "hi:" + arg1;
    }

    public static I1$Greet of() {
        // @method of()LI1$Greet;
        // @declaration an interface's static method of `I1$Greet`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new I1$Greet$1();
    }

    // jarde: no body: the member `name()Ljava/lang/String;` is declared abstract and its declaration carries no Code attribute
    public abstract java.lang.String name();
}
