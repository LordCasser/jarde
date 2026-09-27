// jarde: presentation of `em06/FieldOrder` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em06;

public class FieldOrder extends java.lang.Object {
    private static final java.lang.StringBuilder trace = new java.lang.StringBuilder();

    private static final java.lang.String a = em06.FieldOrder.trace.append("a").toString();

    private static final java.lang.String b = em06.FieldOrder.trace.append("b").toString();

    private static final java.lang.String c = em06.FieldOrder.trace.append("c").toString();

    private static final java.lang.String result = em06.FieldOrder.trace.toString();

    private java.lang.StringBuilder state;

    private int field;

    public FieldOrder() {
        // @method <init>()V
        // @declaration a constructor of `em06.FieldOrder`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.initBuilder(new java.lang.StringBuilder("sb"));
        this.field = this.initField();
        this.state.append(this.field);
        return;
    }

    private void initBuilder(java.lang.StringBuilder arg1) {
        // @method initBuilder(Ljava/lang/StringBuilder;)V
        // @declaration an instance method of `em06.FieldOrder`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        this.state = arg1;
        return;
    }

    private int initField() {
        // @method initField()I
        // @declaration an instance method of `em06.FieldOrder`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return this.state.length();
    }

    public java.lang.String value() {
        // @method value()Ljava/lang/String;
        // @declaration an instance method of `em06.FieldOrder`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.state.toString();
    }

    public static java.lang.String statics() {
        // @method statics()Ljava/lang/String;
        // @declaration a static method of `em06.FieldOrder`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.StringBuilder().append(em06.FieldOrder.a).append(":").append(em06.FieldOrder.b).append(":").append(em06.FieldOrder.c).append(":").append(em06.FieldOrder.result).toString();
    }
}
