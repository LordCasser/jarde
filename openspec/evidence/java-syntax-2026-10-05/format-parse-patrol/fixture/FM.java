public class FM {
    static String fmt(int n, String s){ return String.format("%04d-%s", n, s); }        // String.format varargs+装箱
    static String build(){ StringBuilder sb = new StringBuilder("abc"); sb.append("de").insert(0, "X").replace(1, 2, "Y").deleteCharAt(0); sb.reverse(); sb.setLength(4); return sb.toString(); }  // SB 变异链
    static int parse(String s){ try{ return Integer.parseInt(s.trim()); }catch(NumberFormatException e){ return -1; } }   // parseInt+catch（解析惯用法）
    static int digits(String s){ int n = 0; for(int i = 0; i < s.length(); i++){ if(Character.isDigit(s.charAt(i))){ n++; } } return n; }  // charAt 循环
    static String sub(String s){ return s.substring(1, 3) + s.valueOf(42) + s.toUpperCase().toLowerCase(); }   // substring/valueOf/case 链
    public static void main(String[] a){ System.out.println(""+fmt(7,"x")+"/["+build()+"]/"+parse(" 12 ")+"/"+parse("no")+"/"+digits("a1b2")+"/"+sub("wxyz")); }
}
