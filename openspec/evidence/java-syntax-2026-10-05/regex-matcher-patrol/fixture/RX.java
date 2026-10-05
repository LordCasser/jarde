import java.util.regex.*;
public class RX {
    static final Pattern WORD = Pattern.compile("[a-z]+");            // 静态编译字段（惯用法）
    static int countWords(String s){
        Matcher m = WORD.matcher(s);
        int n = 0;
        while(m.find()){ n++; }
        return n;
    }
    static String firstMatch(String s){
        Matcher m = WORD.matcher(s);
        return m.find() ? m.group() : "-";
    }
    static String groups(String s){
        Matcher m = Pattern.compile("(\\d+)-(\\d+)").matcher(s);
        if(m.matches()){ return m.group(1) + ":" + m.group(2); }
        return "no";
    }
    static String cleaned(String s){ return s.replaceAll("\\s+", " ").trim(); }
    public static void main(String[] a){ System.out.println(""+countWords("ab cd ef")+"/"+firstMatch("12 xy 34")+"/"+groups("7-42")+"/"+groups("x")+"/"+cleaned("  a   b  ")); }
}
