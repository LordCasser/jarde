// jarde: presentation of `UT` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class UT extends java.lang.Object {
    static int 变量;

    static java.lang.String 描述;

    public UT() {
        // @method <init>()V
        // @declaration a constructor of `UT`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int 方法(int arg0) {
        // @method 方法(I)I
        // @declaration a static method of `UT`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 * 2;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `UT`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        内部类 local1 = new 内部类();
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(UT.描述).append("/").append(方法(21)).append("/").append(local1.名字).toString());
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `UT`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        UT.变量 = 1;
        UT.描述 = new java.lang.StringBuilder().append("变量=").append(UT.变量).toString();
    }

    static class 内部类 extends java.lang.Object {
        java.lang.String 名字;

        内部类() {
            // @method <init>()V
            // @declaration a constructor of `UT$内部类`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            this.名字 = "中文";
            return;
        }
    }
}
