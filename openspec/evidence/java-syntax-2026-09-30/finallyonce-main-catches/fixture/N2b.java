public class N2b {
    public static String g(boolean cond) {
        StringBuilder sb = new StringBuilder();
        sb.append("a");
        if (cond) {
            sb.append("b");
        }
        return sb.append("c").toString();
    }
    public static String other() { return "x" + 1; }
    public static void main(String[] args) {
        System.out.println(g(true) + other());
    }
}
