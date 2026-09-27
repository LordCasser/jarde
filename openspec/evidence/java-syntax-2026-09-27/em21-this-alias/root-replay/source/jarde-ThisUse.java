// jarde: presentation of `em21/ThisUse` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em21;

public class ThisUse extends java.lang.Object {
    public int field;

    private int touches;

    public ThisUse() {
        // @method <init>()V
        // @declaration a constructor of `em21.ThisUse`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void inline() {
        // @method inline()V
        // @declaration an instance method of `em21.ThisUse`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.touch();
        this.field = 123;
        return;
    }

    public void checked() {
        // @method checked()V
        // @declaration an instance method of `em21.ThisUse`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        if (java.util.Objects.isNull((java.lang.Object) this)) {
            java.lang.System.out.println("null");
        }
        this.touch();
        this.field = 123;
        return;
    }

    public em21.ThisUse choose() {
        // @method choose()Lem21/ThisUse;
        // @declaration an instance method of `em21.ThisUse`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        em21.ThisUse local1;
        if (this.field == 7) {
            local1 = this;
            java.lang.System.out.print("");
        } else {
            local1 = new em21.ThisUse();
        }
        local1.touch();
        return local1;
    }

    private void touch() {
        // @method touch()V
        // @declaration an instance method of `em21.ThisUse`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        this.touches++;
        return;
    }

    public int touches() {
        // @method touches()I
        // @declaration an instance method of `em21.ThisUse`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.touches;
    }
}
