// jarde: presentation of `Outer$Member` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class Outer$Member extends MemberBase {
    final Outer this$0;

    public Outer$Member(Outer this$0) {
        // @method <init>(LOuter;)V
        // @declaration a constructor of `Outer$Member`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.this$0 = this$0;
        super();
        return;
    }

    public int outerField() {
        // @method outerField()I
        // @declaration an instance method of `Outer$Member`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return Outer.access$000(this.this$0);
    }

    public java.lang.String outerMethod() {
        // @method outerMethod()Ljava/lang/String;
        // @declaration an instance method of `Outer$Member`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return Outer.access$100(this.this$0);
    }

    public java.lang.String outerSuper() {
        // @method outerSuper()Ljava/lang/String;
        // @declaration an instance method of `Outer$Member`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return Outer.access$201(this.this$0);
    }

    public java.lang.String ordinaryThis() {
        // @method ordinaryThis()Ljava/lang/String;
        // @declaration an instance method of `Outer$Member`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.value();
    }
}
