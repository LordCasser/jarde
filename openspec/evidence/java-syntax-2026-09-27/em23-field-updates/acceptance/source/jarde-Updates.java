// jarde: presentation of `em23/Updates` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em23;

public class Updates extends java.lang.Object {
    public int instanceField;

    public static int staticField;

    public static java.lang.String result;

    public Updates() {
        // @method <init>()V
        // @declaration a constructor of `em23.Updates`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.instanceField = 1;
        return;
    }

    public void increment() {
        // @method increment()V
        // @declaration an instance method of `em23.Updates`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.instanceField += 1;
        return;
    }

    public void decrement() {
        // @method decrement()V
        // @declaration an instance method of `em23.Updates`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        em23.Updates.staticField = em23.Updates.staticField - 1;
        return;
    }

    public void append(java.lang.String arg1) {
        // @method append(Ljava/lang/String;)V
        // @declaration an instance method of `em23.Updates`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        em23.Updates.result = new java.lang.StringBuilder().append(em23.Updates.result).append(arg1).append('_').toString();
        return;
    }

    public int plusTwo(int arg1) {
        // @method plusTwo(I)I
        // @declaration an instance method of `em23.Updates`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        arg1 = arg1 + 2;
        return arg1;
    }

    public int next(int arg1) {
        // @method next(I)I
        // @declaration an instance method of `em23.Updates`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        arg1 = arg1 + 1;
        return arg1;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `em23.Updates`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        em23.Updates.staticField = 1;
        em23.Updates.result = "";
    }
}
