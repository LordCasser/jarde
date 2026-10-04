// jarde: presentation of `AnonymousMemberBase` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class AnonymousMemberBase extends java.lang.Object {
    private static final java.lang.StringBuilder EVENTS = new java.lang.StringBuilder();

    public AnonymousMemberBase() {
        // @method <init>()V
        // @declaration a constructor of `AnonymousMemberBase`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static void event(java.lang.String value) {
        // @method event(Ljava/lang/String;)V
        // @declaration a static method of `AnonymousMemberBase`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        if (AnonymousMemberBase.EVENTS.length() > 0) {
            AnonymousMemberBase.EVENTS.append(',');
        }
        AnonymousMemberBase.EVENTS.append(value);
        return;
    }

    private static AnonymousMemberBase$Outer outer() {
        // @method outer()LAnonymousMemberBase$Outer;
        // @declaration a static method of `AnonymousMemberBase`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        event("outer");
        return new AnonymousMemberBase$Outer();
    }

    private static AnonymousMemberBase$Outer nullOuter() {
        // @method nullOuter()LAnonymousMemberBase$Outer;
        // @declaration a static method of `AnonymousMemberBase`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        event("nullOuter");
        return null;
    }

    private static int sideEffect() {
        // @method sideEffect()I
        // @declaration a static method of `AnonymousMemberBase`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        event("argument");
        return 7;
    }

    private static AnonymousMemberBase$Outer$Base normal() {
        // @method normal()LAnonymousMemberBase$Outer$Base;
        // @declaration a static method of `AnonymousMemberBase`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        AnonymousMemberBase$Outer outer = outer();
        // @bytecode 4
        // the instruction at BCI 4 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 7
        // the instruction at BCI 7 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 9
        // the instruction at BCI 9 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 10 13 8
        // the copy at BCI 9 has no proved local assignment
        // @bytecode 20 17 14 8
        // the copy at BCI 7 has no proved local assignment
    }

    private static AnonymousMemberBase$Outer$Base nullPath() {
        // @method nullPath()LAnonymousMemberBase$Outer$Base;
        // @declaration a static method of `AnonymousMemberBase`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        AnonymousMemberBase$Outer outer = nullOuter();
        // @bytecode 4
        // the instruction at BCI 4 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 7
        // the instruction at BCI 7 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 9
        // the instruction at BCI 9 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 10 13 8
        // the copy at BCI 9 has no proved local assignment
        // @bytecode 20 17 14 8
        // the copy at BCI 7 has no proved local assignment
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `AnonymousMemberBase`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        AnonymousMemberBase$Outer$Base value = normal();
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("normal=").append(value.render()).append(";events=").append((java.lang.Object) AnonymousMemberBase.EVENTS).toString());
        AnonymousMemberBase.EVENTS.setLength(0);
        try {
            nullPath();
            throw new java.lang.AssertionError((java.lang.Object) "expected null receiver failure");
        } catch (java.lang.NullPointerException expected) {
            java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("null=").append((java.lang.Object) AnonymousMemberBase.EVENTS).toString());
            return;
        }
    }

    static void access$000(java.lang.String x0) {
        // @method access$000(Ljava/lang/String;)V
        // @declaration a static method of `AnonymousMemberBase`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        event(x0);
        return;
    }
}
