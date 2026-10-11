// jarde: presentation of `cf11/UpstreamSequentialLoops2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public final class UpstreamSequentialLoops2 extends java.lang.Object {
    private static char[] lowercases;

    public UpstreamSequentialLoops2() {
        // @method <init>()V
        // @declaration a constructor of `cf11.UpstreamSequentialLoops2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String asciiToLowerCase(java.lang.String s) {
        // @method asciiToLowerCase(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `cf11.UpstreamSequentialLoops2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        char[] c;
        int i;
        c = null;
        i = s.length();
        while (i-- > 0) {
            char c1 = s.charAt(i);
            if (c1 <= 127) {
                char c2 = cf11.UpstreamSequentialLoops2.lowercases[c1];
                if (c1 != c2) {
                    c = s.toCharArray();
                    c[i] = c2;
                    break;
                }
            }
        }
        while (i-- > 0) {
            if (c[i] <= 127) {
                c[i] = cf11.UpstreamSequentialLoops2.lowercases[c[i]];
            }
        }
        return c == null ? s : new java.lang.String(c);
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `cf11.UpstreamSequentialLoops2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        cf11.UpstreamSequentialLoops2.lowercases = new char[]{'a'};
    }
}
