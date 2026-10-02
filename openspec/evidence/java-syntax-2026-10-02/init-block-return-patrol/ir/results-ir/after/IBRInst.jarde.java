// jarde: presentation of `IBRInst` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class IBRInst extends java.lang.Object {
    int x;

    int y;

    IBRInst() {
        // @method <init>()V
        // @declaration a constructor of `IBRInst`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.x = 4;
        if (this.x < 0) {
            throw new java.lang.IllegalStateException("never");
        } else {
            this.y = this.x * 2;
            return;
        }
    }

    IBRInst(int arg1) {
        // @method <init>(I)V
        // @declaration a constructor of `IBRInst`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.x = 4;
        if (this.x < 0) {
            throw new java.lang.IllegalStateException("never");
        } else {
            this.y = this.x * 2;
            this.y = this.y + arg1;
            return;
        }
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `IBRInst`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        IBRInst local1 = new IBRInst();
        IBRInst local2 = new IBRInst(10);
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(local1.x).append(":").append(local1.y).append(":").append(local2.x).append(":").append(local2.y).toString());
        return;
    }
}
