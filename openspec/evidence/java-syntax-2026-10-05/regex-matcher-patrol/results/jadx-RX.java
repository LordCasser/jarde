package defpackage;

/* JADX INFO: loaded from: RX.class */
public class RX {
    static final java.util.regex.Pattern WORD = java.util.regex.Pattern.compile("[a-z]+");

    static int countWords(java.lang.String str) {
        int i = 0;
        while (WORD.matcher(str).find()) {
            i++;
        }
        return i;
    }

    static java.lang.String firstMatch(java.lang.String str) {
        java.util.regex.Matcher matcher = WORD.matcher(str);
        return matcher.find() ? matcher.group() : "-";
    }

    static java.lang.String groups(java.lang.String str) {
        java.util.regex.Matcher matcher = java.util.regex.Pattern.compile("(\\d+)-(\\d+)").matcher(str);
        return matcher.matches() ? matcher.group(1) + ":" + matcher.group(2) : "no";
    }

    static java.lang.String cleaned(java.lang.String str) {
        return str.replaceAll("\\s+", " ").trim();
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + countWords("ab cd ef") + "/" + firstMatch("12 xy 34") + "/" + groups("7-42") + "/" + groups("x") + "/" + cleaned("  a   b  "));
    }
}
