public class ES extends java.lang.Object {
    public ES() {
        super();
        return;
    }

    static java.lang.String tabNewline() {
        return "a\tb\nc";
    }

    static java.lang.String quotes() {
        return "say \"hi\" and 'bye'";
    }

    static java.lang.String backslash() {
        return "C:\\dir\\file";
    }

    static java.lang.String octal() {
        return "xz";
    }

    static java.lang.String unicode() {
        return "中文";
    }

    static java.lang.String mixed() {
        return "\r\n\t\\\"é";
    }

    public static void main(java.lang.String[] arg0) {
        java.lang.System.out.println("" + tabNewline().replace('\n', '|') + "/" + quotes() + "/" + backslash() + "/" + octal() + "/" + unicode() + "/" + mixed().replace('\r', 'R').replace('\n', 'N'));
        return;
    }
}
