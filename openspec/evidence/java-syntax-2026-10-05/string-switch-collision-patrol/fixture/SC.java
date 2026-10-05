public class SC {
    static String pick(String k){
        switch(k){
            case "1012": return "a";        // 与下一行 hashCode 碰撞（1507456）
            case "14669600": return "b";    // javac 必须发二次 equals 检查
            case "plain": return "c";       // 无碰撞对照
            default: return "?";
        }
    }
    public static void main(String[] a){ System.out.println(""+pick("1012")+"/"+pick("14669600")+"/"+pick("plain")+"/"+pick("x")); }
}
