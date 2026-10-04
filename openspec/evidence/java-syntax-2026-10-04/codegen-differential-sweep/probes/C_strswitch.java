public class C_strswitch {
    static int pick(String s){ switch(s){ case "a": return 1; case "b": return 2; case "c": return 3; default: return 0; } }
    public static void main(String[] a){ System.out.println(pick("a")+pick("b")+pick("z")); }
}
