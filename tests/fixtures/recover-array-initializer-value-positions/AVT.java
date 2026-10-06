public class AVT {
    static String[][] s = new String[2][];
    static long[][] l = new long[2][];
    static double[][] d = new double[2][];
    static boolean[][] b = new boolean[2][];
    static char[][] c = new char[2][];
    static void setS() { s[0] = new String[]{"a", "b"}; }
    static void setL() { l[0] = new long[]{1L}; }
    static void setD() { d[0] = new double[]{1.5}; }
    static void setB() { b[0] = new boolean[]{true}; }
    static void setC() { c[0] = new char[]{'x'}; }
    static String idxS() { return new String[]{"a", "b"}[1]; }
    static long idxL() { return new long[]{1L}[0]; }
    static double idxD() { return new double[]{1.5}[0]; }
    static boolean idxB() { return new boolean[]{true}[0]; }
    static char idxC() { return new char[]{'x'}[0]; }
    static int lenD() { return new double[]{1.5}.length; }
    public static void main(String[] a) {
        setS(); setL(); setD(); setB(); setC();
        System.out.print(idxS()); System.out.print("/");
        System.out.print(idxL()); System.out.print("/");
        System.out.print(idxD()); System.out.print("/");
        System.out.print(idxB()); System.out.print("/");
        System.out.print(idxC()); System.out.print("/");
        System.out.print(lenD()); System.out.print("/");
        System.out.print(s[0][1]); System.out.print("/");
        System.out.print(l[0][0]); System.out.print("/");
        System.out.print(d[0][0]); System.out.print("/");
        System.out.print(b[0][0]); System.out.print("/");
        System.out.println(c[0][0]);
    }
}
