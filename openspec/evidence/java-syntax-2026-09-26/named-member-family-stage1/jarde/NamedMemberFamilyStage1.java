// jarde: presentation of `NamedMemberFamilyStage1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class NamedMemberFamilyStage1 extends java.lang.Object {
    int state;

    private int secret;

    NamedMemberFamilyStage1(int state, int secret) {
        // @method <init>(II)V
        // @declaration a constructor of `NamedMemberFamilyStage1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.state = state;
        this.secret = secret;
        return;
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `NamedMemberFamilyStage1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        NamedMemberFamilyStage1 outer = new NamedMemberFamilyStage1(10, 11);
        NamedMemberFamilyStage1 other = new NamedMemberFamilyStage1(20, 22);
        java.io.PrintStream saved0 = java.lang.System.out;
        // @bytecode 27
        // the instruction at BCI 27 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 30
        // the instruction at BCI 30 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 32
        // the instruction at BCI 32 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 33 36 31
        // the value at BCI 33 comes from an Duplicate at BCI 32, which produces no expression this subset writes
        // @bytecode 44 24 41 37 31 40
        // the value at BCI 44 comes from an Duplicate at BCI 30, which produces no expression this subset writes
        java.lang.System.out.println(other.state);
        return;
    }

    static int access$000(NamedMemberFamilyStage1 x0) {
        // @method access$000(LNamedMemberFamilyStage1;)I
        // @declaration a static method of `NamedMemberFamilyStage1`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        return x0.secret;
    }
}
