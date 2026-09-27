// jarde: presentation of `BitmaskAudit` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class BitmaskAudit extends java.lang.Object {
    private static final int MASK = 2;

    private static int reads;

    public BitmaskAudit() {
        // @method <init>()V
        // @declaration a constructor of `BitmaskAudit`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static int observe(int arg0) {
        // @method observe(I)I
        // @declaration a static method of `BitmaskAudit`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        BitmaskAudit.reads = BitmaskAudit.reads + 1;
        return arg0;
    }

    public static int select(int arg0) {
        // @method select(I)I
        // @declaration a static method of `BitmaskAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if ((observe(arg0) & 2) != 0) {
            return 11;
        } else {
            return 22;
        }
    }

    public static int selectZeroMask(int arg0) {
        // @method selectZeroMask(I)I
        // @declaration a static method of `BitmaskAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if ((observe(arg0) & 2) == 0) {
            return 33;
        } else {
            return 44;
        }
    }

    private static java.lang.String measured(int arg0) {
        // @method measured(I)Ljava/lang/String;
        // @declaration a static method of `BitmaskAudit`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        BitmaskAudit.reads = 0;
        int local1 = select(arg0);
        int local2 = BitmaskAudit.reads;
        BitmaskAudit.reads = 0;
        int local3 = selectZeroMask(arg0);
        return new java.lang.StringBuilder().append(local1).append(":").append(local2).append(",").append(local3).append(":").append(BitmaskAudit.reads).toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `BitmaskAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int[] local2;
        int[] local1 = new int[]{0, 1, 2, 3, -2147483648, -2147483646};
        local2 = local1;
        for (int local5 : local2) {
            java.lang.System.out.println("" + local5 + "=" + measured(local5));
        }
        return;
    }
}
